use std::borrow::Cow;

use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context};
use sea_orm::prelude::*;
use sea_orm::ActiveValue;

use pegasus_common::database::entities;

#[derive(Clone)]
pub struct ForwardingBotService {
    db: DatabaseConnection,
}

impl ForwardingBotService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub trait IForwardingBotService {
    ///
    /// Create a new bot record
    ///
    /// # Arguments
    ///
    /// * `cx`: context
    /// * `bot_token`: bot token
    /// * `target_chat_id`: target chat id
    ///
    /// returns: `Result<Model, Error>`
    ///
    async fn create_bot_record(
        &self,
        cx: &Context,
        bot_token: String,
        target_chat_id: i64,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;

    async fn get_bot_record_by_token(
        &self,
        cx: &Context,
        bot_token: String,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;
}

impl IForwardingBotService for ForwardingBotService {
    async fn create_bot_record(
        &self,
        cx: &Context,
        bot_token: String,
        target_chat_id: i64,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingBotService.create_bot_record")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let bot = entities::pm_forwarding_bot::ActiveModel {
            bot_token: ActiveValue::Set(bot_token),
            target_chat_id: ActiveValue::Set(target_chat_id),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            err
        })?;

        Ok(bot)
    }

    async fn get_bot_record_by_token(
        &self,
        cx: &Context,
        bot_token: String,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingBotService.get_bot_record_by_token")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let bot = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::BotToken.eq(bot_token))
            .one(&self.db)
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                err
            })?
            .ok_or_else(|| {
                cx.span().set_status(Status::Error {
                    description: Cow::from("Bot not found"),
                });
                anyhow::anyhow!("Bot not found")
            })?;

        Ok(bot)
    }
}
