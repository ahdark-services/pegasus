use std::borrow::Cow;

use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use opentelemetry::{global, Context};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::Url;
use sea_orm::prelude::*;
use sea_orm::ActiveValue;
use teloxide::prelude::*;

use pegasus_common::database::entities;
use pegasus_common::settings::Settings;

#[derive(Clone)]
pub struct ForwardingBotService {
    db: DatabaseConnection,
    settings: Settings,
}

impl ForwardingBotService {
    pub fn new(db: DatabaseConnection, settings: Settings) -> Self {
        Self { db, settings }
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
    /// * `user_id`: telegram user id
    ///
    /// returns: `Result<Model, Error>`
    ///
    async fn create_bot_record(
        &self,
        cx: &Context,
        bot_token: String,
        target_chat_id: i64,
        user_id: u64,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;

    ///
    /// Get bot record by token
    ///
    /// # Arguments
    ///
    /// * `cx`: context
    /// * `bot_token`: bot token
    ///
    /// returns: `Result<Model, Error>` bot record model
    ///
    async fn get_bot_record_by_token(
        &self,
        cx: &Context,
        bot_token: String,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;

    ///
    /// Check if token exists
    ///
    /// # Arguments
    ///
    /// * `cx`: context
    /// * `bot_token`: bot token
    ///
    /// returns: `Result<bool, Error>` true if token exists
    ///
    async fn check_token_exist(&self, cx: &Context, bot_token: String) -> anyhow::Result<bool>;

    ///
    /// Initialize bot, log out the bot and set the webhook to local api server
    ///
    /// # Arguments
    ///
    /// * `cx`: context
    /// * `bot`: bot id
    ///
    /// returns: `Result<(), Error>`
    ///
    async fn initialize_bot(&self, cx: &Context, bot: i64) -> anyhow::Result<()>;

    ///
    /// List bots by telegram user id
    ///
    /// # Arguments
    ///
    /// * `cx`: context
    /// * `telegram_user_id`: telegram user id
    ///
    /// returns: `Result<Vec<Model, Global>, Error>` list of bot records
    ///
    async fn list_bots(
        &self,
        cx: &Context,
        telegram_user_id: u64,
    ) -> anyhow::Result<Vec<entities::pm_forwarding_bot::Model>>;
}

/// Generate a random webhook secret
fn random_webhook_secret() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

impl IForwardingBotService for ForwardingBotService {
    async fn create_bot_record(
        &self,
        cx: &Context,
        bot_token: String,
        target_chat_id: i64,
        user_id: u64,
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
            bot_webhook_secret: ActiveValue::Set(random_webhook_secret()),
            target_chat_id: ActiveValue::Set(target_chat_id),
            telegram_user_refer: ActiveValue::Set(user_id as i64),
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

        self.initialize_bot(&cx, bot.id).await?;

        Ok(bot)
    }

    async fn check_token_exist(&self, cx: &Context, bot_token: String) -> anyhow::Result<bool> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingBotService.check_token_exist")
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
            })?;

        Ok(bot.is_some())
    }

    async fn initialize_bot(&self, cx: &Context, bot_id: i64) -> anyhow::Result<()> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingBotService.initialize_bot")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let bot = entities::pm_forwarding_bot::Entity::find_by_id(bot_id)
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

        let api_url = self
            .settings
            .telegram_bot
            .clone()
            .unwrap()
            .api_url
            .unwrap_or("https://api.telegram.org/".into());

        let client = Bot::new(&bot.bot_token).set_api_url(Url::parse(&api_url)?);
        match client.log_out().await {
            Ok(_) => log::debug!("Bot logged out, id: {}", bot.id),
            Err(err) => {
                cx.span().record_error(&err);
                // ignore the error
            }
        }

        let webhook_url = format!(
            "http://pm-bot-forwarding-handler:8080/webhook/{}",
            bot.bot_token
        );
        client
            .set_webhook(Url::parse(&webhook_url)?)
            .secret_token(&bot.bot_webhook_secret)
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                err
            })?;

        Ok(())
    }

    async fn list_bots(
        &self,
        cx: &Context,
        telegram_user_id: u64,
    ) -> anyhow::Result<Vec<entities::pm_forwarding_bot::Model>> {
        let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/services");
        let cx = cx.with_span(
            tracer
                .span_builder("ForwardingBotService.list_bots")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .start_with_context(&tracer, cx),
        );

        let bots = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::TelegramUserRefer.eq(telegram_user_id))
            .all(&self.db)
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                err
            })?;

        Ok(bots)
    }
}
