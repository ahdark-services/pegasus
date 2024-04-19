use std::borrow::Cow;

use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::Context;
use reqwest::Url;
use sea_orm::prelude::*;
use sea_orm::ActiveValue;
use teloxide::prelude::*;
use teloxide::types::{MessageId, UpdateKind};

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
            .ok_or_else(|| anyhow::anyhow!("Missing chat"))?
            .to_owned();

        let message_id = match &update.kind {
            UpdateKind::Message(m) => m.id,
            _ => return Ok(()), // ignore non-message updates
        };

        let client = self.new_bot_client(&bot.bot_token)?;

        let r = if chat.id.0 == bot.target_chat_id {
            self.handle_target_chat_message(bot, update).await
        } else if chat.is_private() {
            self.handle_forward_message(bot, update).await
        } else {
            log::debug!("Ignoring message from chat {}", chat.id.0);
            return Ok(());
        };

        match r {
            Err(err) => {
                log::error!("Error handling message: {}, chat_id: {}", err, chat.id.0);
                client
                    .send_message(chat.id, format!("Error handling message: {}", err))
                    .reply_to_message_id(message_id)
                    .await?;
            }
            Ok(_) => {
                log::debug!("Message handled successfully, chat_id: {}", chat.id.0);
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

        let bot = self.new_bot_client(&bot_info.bot_token)?;

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

                // forward message to target chat
                let msg_forward = bot
                    .send_message(
                        ChatId(bot_info.target_chat_id),
                        combine_forwarding_content(
                            &message
                                .from()
                                .map(|from| from.first_name.as_str())
                                .unwrap_or("Unknown"),
                            chat.id.0,
                            message.id.0,
                            message.text().unwrap_or("Empty Message"),
                        ),
                    )
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .await?;

                // store message to database
                entities::pm_forwarding_message::ActiveModel {
                    bot_id: ActiveValue::Set(bot_info.id),
                    telegram_chat_id: ActiveValue::Set(chat.id.0),
                    telegram_message_id: ActiveValue::Set(message.id.0),
                    forward_telegram_message_id: ActiveValue::Set(msg_forward.id.0),
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
        let cx = Context::current();

        match update.kind {
            UpdateKind::Message(ref message) => {
                let bot = Bot::new(bot_info.bot_token);
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
                    message.text().ok_or_else(|| {
                        cx.span().set_status(Status::Error {
                            description: Cow::from("Missing message text"),
                        });
                        anyhow::anyhow!("Missing message text")
                    })?,
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

fn combine_forwarding_content(from: &str, chat_id: i64, message_id: i32, text: &str) -> String {
    format!(
        r#"
        From: {}
        Chat ID: <a href="tg://user?id={}">{}</a>
        Message ID: <code>{}</code>
        
        {}
        "#,
        from, chat_id, chat_id, message_id, text
    )
    .lines()
    .map(|line| line.trim_start())
    .collect::<Vec<_>>()
    .join("\n")
    .trim()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_combine_forwarding_content() {
        let from = "test";
        let chat_id = 123;
        let message_id = 456;
        let text = "test message";

        let content = combine_forwarding_content(from, chat_id, message_id, text);
        assert_eq!(
            content,
            "From: test\nChat ID: <a href=\"tg://user?id=123\">123</a>\nMessage ID: <code>456</code>\n\ntest message"
        );
    }
}
