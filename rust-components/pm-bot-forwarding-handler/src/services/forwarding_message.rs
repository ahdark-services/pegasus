use std::borrow::Cow;

use opentelemetry::trace::{SpanKind, Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context};
use reqwest::Url;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel, TransactionTrait};
use teloxide::prelude::*;
use teloxide::types::{MessageId, UpdateKind};

use pegasus_common::database::entities;
use pegasus_common::settings::Settings;

#[derive(Clone)]
pub struct ForwardingMessageService {
    db: DatabaseConnection,
    settings: Settings,
}

impl ForwardingMessageService {
    pub fn new(db: DatabaseConnection, settings: Settings) -> Self {
        Self { db, settings }
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
    async fn handle_update_income(
        &self,
        cx: &Context,
        bot_id: i64,
        update: Update,
    ) -> anyhow::Result<()>;
}

impl IForwardingMessageService for ForwardingMessageService {
    async fn handle_update_income(
        &self,
        cx: &Context,
        bot_id: i64,
        update: Update,
    ) -> anyhow::Result<()> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingMessageService.handle_update_income")
                .with_kind(SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let bot = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::Id.eq(bot_id))
            .one(&self.db)
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                err
            })?
            .ok_or_else(|| anyhow::anyhow!("Bot record not found"))
            .map_err(|err| {
                cx.span().record_error(err.as_ref());
                err
            })?;

        let chat = update
            .chat()
            .ok_or_else(|| anyhow::anyhow!("Missing chat"))
            .map_err(|err| {
                cx.span().record_error(err.as_ref());
                err
            })?;

        if chat.id.0 == bot.target_chat_id {
            self.handle_target_chat_message(&cx, bot, update).await
        } else {
            self.handle_forward_message(&cx, bot, update).await
        }
        .map_err(|err| {
            cx.span().record_error(err.as_ref());
            err
        })
    }
}

impl ForwardingMessageService {
    async fn handle_forward_message(
        &self,
        cx: &Context,
        bot_info: entities::pm_forwarding_bot::Model,
        update: Update,
    ) -> anyhow::Result<()> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingMessageService.handle_forward_message")
                .with_kind(SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let chat = update
            .chat()
            .ok_or_else(|| {
                cx.span().set_status(Status::Error {
                    description: Cow::from("Missing chat"),
                });
                anyhow::anyhow!("Missing chat")
            })?
            .clone();

        match update.kind {
            UpdateKind::Message(ref message) => {
                let txn = self.db.begin().await?;

                // store message to database
                let message_entity = entities::pm_forwarding_message::ActiveModel {
                    bot_id: ActiveValue::Set(bot_info.id),
                    telegram_chat_id: ActiveValue::Set(chat.id.0),
                    telegram_message_id: ActiveValue::Set(message.id.0),
                    ..Default::default()
                }
                .save(&txn)
                .await
                .map_err(|err| {
                    cx.span().record_error(&err);
                    err
                })?;

                // forward message to target chat
                let api_url = self
                    .settings
                    .telegram_bot
                    .clone()
                    .unwrap()
                    .api_url
                    .unwrap_or("https://api.telegram.org/".into());
                let bot = Bot::new(bot_info.bot_token).set_api_url(Url::parse(&api_url)?);
                let msg = bot
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
                    .await
                    .map_err(|err| {
                        cx.span().record_error(&err);
                        err
                    })?;

                // update message with forwarded message id
                let mut message_active_model = message_entity.into_active_model();
                message_active_model.forward_telegram_message_id = ActiveValue::Set(msg.id.0);
                message_active_model.update(&txn).await.map_err(|err| {
                    cx.span().record_error(&err);
                    err
                })?;

                txn.commit().await.map_err(|err| {
                    cx.span().record_error(&err);
                    err
                })?;
            }
            _ => {
                cx.span().set_status(Status::Error {
                    description: Cow::from("Unsupported update kind"),
                });
                return Err(anyhow::anyhow!("Unsupported update kind"));
            }
        }

        Ok(())
    }

    async fn handle_target_chat_message(
        &self,
        cx: &Context,
        bot_info: entities::pm_forwarding_bot::Model,
        update: teloxide::types::Update,
    ) -> anyhow::Result<()> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingMessageService.handle_target_chat_message")
                .with_kind(SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        match update.kind {
            UpdateKind::Message(ref message) => {
                let bot = Bot::new(bot_info.bot_token);
                let reply_id = message
                    .reply_to_message()
                    .map(|msg| msg.id.0)
                    .ok_or_else(|| {
                        cx.span().set_status(Status::Error {
                            description: Cow::from("Missing reply message"),
                        });
                        anyhow::anyhow!("Missing reply message")
                    })?;

                // find original message
                let message_entity = entities::pm_forwarding_message::Entity::find()
                    .filter(
                        entities::pm_forwarding_message::Column::ForwardTelegramMessageId
                            .eq(reply_id),
                    )
                    .one(&self.db)
                    .await
                    .map_err(|err| {
                        cx.span().record_error(&err);
                        err
                    })?
                    .ok_or_else(|| anyhow::anyhow!("Message not found"))
                    .map_err(|err| {
                        cx.span().record_error(err.as_ref());
                        err
                    })?;

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
                .await
                .map_err(|err| {
                    cx.span().record_error(&err);
                    err
                })?;
            }
            _ => {
                cx.span().set_status(Status::Error {
                    description: Cow::from("Unsupported update kind"),
                });
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
        Chat ID: <code>{}</code>
        Message ID: <code>{}</code>
        
        {}
        "#,
        from, chat_id, message_id, text
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
            "From: test\nChat ID: <code>123</code>\nMessage ID: <code>456</code>\n\ntest message"
        );
    }
}
