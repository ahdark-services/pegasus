use reqwest::Url;
use sea_orm::prelude::*;
use sea_orm::ActiveValue;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MediaKind, MessageId, MessageKind, UpdateKind};

use pegasus_common::database::entities;
use pegasus_common::settings::Settings;

#[derive(Clone, Debug)]
pub struct ForwardingMessageService {
    db: DatabaseConnection,
    settings: Settings,
}

impl ForwardingMessageService {
    pub fn new(db: DatabaseConnection, settings: Settings) -> Self {
        Self { db, settings }
    }

    #[tracing::instrument(err)]
    fn new_bot_client(&self, token: &str) -> anyhow::Result<Bot> {
        let api_url = self
            .settings
            .telegram_bot
            .clone()
            .unwrap()
            .api_url
            .unwrap_or("https://api.telegram.org/".into());

        let client = Bot::new(token).set_api_url(Url::parse(&api_url)?);
        Ok(client)
    }
}

pub trait IForwardingMessageService {
    ///
    /// Handle update income
    ///
    /// # Arguments
    ///
    /// * `bot_id`: bot id stored in the database
    /// * `update`: telegram bot update content
    ///
    /// returns: `Result<(), Error>`
    ///
    async fn handle_update_income(&self, bot_id: i64, update: Update) -> anyhow::Result<()>;
}

impl IForwardingMessageService for ForwardingMessageService {
    #[tracing::instrument(err)]
    async fn handle_update_income(&self, bot_id: i64, update: Update) -> anyhow::Result<()> {
        let bot = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::Id.eq(bot_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Bot record not found"))?;

        let chat = update
            .chat()
            .ok_or_else(|| anyhow::anyhow!("Missing chat"))?;

        let message_id = match &update.kind {
            UpdateKind::Message(m) => m.id,
            _ => {
                // ignore non-message updates
                log::debug!("Ignoring non-message update from chat {}", &chat.id.0);
                return Ok(());
            }
        };

        let client = self.new_bot_client(&bot.bot_token)?;

        let r = if chat.id.0 == bot.target_chat_id {
            log::debug!("Handling message reply from target chat {}", &chat.id.0);
            self.handle_target_chat_message(bot, update.clone()).await
        } else if chat.is_private() {
            log::debug!("Handling message from chat {}", &chat.id.0);
            self.handle_forward_message(bot, update.clone()).await
        } else {
            log::debug!("Ignoring message from chat {}", &chat.id.0);
            return Ok(());
        };

        match r {
            Err(err) => {
                log::error!("Error handling message: {}, chat_id: {}", err, &chat.id.0);
                client
                    .send_message(chat.id, format!("Error handling message: {}", err))
                    .reply_to_message_id(message_id)
                    .await?;
            }
            Ok(_) => {
                log::debug!("Message handled successfully, chat_id: {}", &chat.id.0);
                client
                    .send_message(chat.id, "Message sent")
                    .reply_to_message_id(message_id)
                    .await?;
            }
        }

        Ok(())
    }
}

impl ForwardingMessageService {
    #[tracing::instrument(err)]
    async fn handle_forward_message(
        &self,
        bot_info: entities::pm_forwarding_bot::Model,
        update: Update,
    ) -> anyhow::Result<()> {
        let chat = update
            .chat()
            .ok_or_else(|| anyhow::anyhow!("Missing chat"))?
            .clone();

        tracing::debug!("Creating bot client");

        let bot = self
            .new_bot_client(&bot_info.bot_token)
            .map_err(|err| anyhow::anyhow!("Error creating bot client: {}", err))?;

        tracing::debug!("Matching update kind & media kind");

        match update.kind {
            UpdateKind::Message(ref message) => {
                if message.from().is_none() || message.from().unwrap().is_bot {
                    log::debug!(
                        "Ignoring message from chat {}, sender is unknown",
                        chat.id.0
                    );
                    return Ok(());
                }

                if message.text().is_none() || message.text().unwrap().starts_with("/") {
                    log::debug!("Ignoring command message from chat {}", chat.id.0);
                    return Ok(());
                }

                let meta = forwarding_meta(
                    &message
                        .from()
                        .map(|from| {
                            format!(
                                "{}{}",
                                from.first_name,
                                if from.last_name.is_some() {
                                    format!(" {}", from.last_name.as_ref().unwrap())
                                } else {
                                    "".into()
                                }
                            )
                        })
                        .unwrap_or("Unknown".into()),
                    chat.id.0,
                    message.id.0,
                );

                let msg_forward_id = match &message.kind {
                    MessageKind::Common(common_message) => match &common_message.media_kind {
                        MediaKind::Text(media) => {
                            bot.send_message(
                                ChatId(bot_info.target_chat_id),
                                format!("{}\n\n{}", meta, media.text),
                            )
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .await?
                            .id
                        }
                        _ => {
                            let file = match &common_message.media_kind {
                                MediaKind::Animation(m) => Some(&m.animation.file),
                                MediaKind::Audio(m) => Some(&m.audio.file),
                                MediaKind::Contact(_) => None,
                                MediaKind::Document(m) => Some(&m.document.file),
                                MediaKind::Game(_) => None,
                                MediaKind::Venue(_) => None,
                                MediaKind::Location(_) => None,
                                MediaKind::Photo(m) => Some(&m.photo.last().unwrap().file),
                                MediaKind::Poll(_) => None,
                                MediaKind::Sticker(m) => Some(&m.sticker.file),
                                MediaKind::Text(_) => {
                                    unreachable!("Text message already handled")
                                }
                                MediaKind::Video(m) => Some(&m.video.file),
                                MediaKind::VideoNote(_) => None,
                                MediaKind::Voice(m) => Some(&m.voice.file),
                                MediaKind::Migration(_) => None,
                            }
                            .ok_or_else(|| anyhow::anyhow!("Unsupported media kind"))?;

                            bot.send_document(
                                ChatId(bot_info.target_chat_id),
                                InputFile::file_id(&file.id),
                            )
                            .await?
                            .id
                        }
                    },
                    _ => {
                        return Err(anyhow::anyhow!("Unsupported message kind"));
                    }
                };

                tracing::debug!(
                    name = "Storing message to database",
                    message_id = msg_forward_id.0,
                );

                // store message to database
                entities::pm_forwarding_message::ActiveModel {
                    bot_id: ActiveValue::Set(bot_info.id),
                    telegram_chat_id: ActiveValue::Set(chat.id.0),
                    telegram_message_id: ActiveValue::Set(message.id.0),
                    forward_telegram_message_id: ActiveValue::Set(msg_forward_id.0),
                    ..Default::default()
                }
                .save(&self.db)
                .await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported update kind"));
            }
        }

        Ok(())
    }

    #[tracing::instrument(err)]
    async fn handle_target_chat_message(
        &self,
        bot_info: entities::pm_forwarding_bot::Model,
        update: Update,
    ) -> anyhow::Result<()> {
        match update.kind {
            UpdateKind::Message(ref message) => {
                let bot = self
                    .new_bot_client(&bot_info.bot_token)
                    .map_err(|err| anyhow::anyhow!("Error creating bot client: {}", err))?;

                let reply_id = message
                    .reply_to_message()
                    .map(|msg| msg.id.0)
                    .ok_or_else(|| anyhow::anyhow!("Missing reply message"))?;

                // find original message
                let message_entity = entities::pm_forwarding_message::Entity::find()
                    .filter(
                        entities::pm_forwarding_message::Column::ForwardTelegramMessageId
                            .eq(reply_id),
                    )
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Message not found"))?;

                // reply to original message
                bot.send_message(
                    ChatId(message_entity.telegram_chat_id),
                    message
                        .text()
                        .ok_or_else(|| anyhow::anyhow!("Missing message text"))?,
                )
                .reply_to_message_id(MessageId(message_entity.telegram_message_id))
                .await?;
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported update kind"));
            }
        }

        Ok(())
    }
}

fn forwarding_meta(from: &str, chat_id: i64, message_id: i32) -> String {
    format!(
        r#"
        From: {}
        Chat ID: <a href="tg://user?id={}">{}</a>
        Message ID: <code>{}</code>
        "#,
        from, chat_id, chat_id, message_id
    )
    .lines()
    .map(|line| line.trim_start())
    .collect::<Vec<_>>()
    .join("\n")
    .trim()
    .to_string()
}
