use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::Url;
use sea_orm::prelude::*;
use sea_orm::ActiveValue;
use teloxide::prelude::*;
use tracing::span;

use pegasus_common::database::entities;
use pegasus_common::settings::Settings;

#[derive(Clone, Debug)]
pub struct ForwardingBotService {
    db: DatabaseConnection,
    settings: Settings,
}

impl ForwardingBotService {
    pub fn new(db: DatabaseConnection, settings: Settings) -> Self {
        Self { db, settings }
    }

    /// Create a new bot client with the given token
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

pub trait IForwardingBotService {
    ///
    /// Create a new bot record
    ///
    /// # Arguments
    ///
    /// * `bot_token`: bot token
    /// * `target_chat_id`: target chat id
    /// * `user_id`: telegram user id
    ///
    /// returns: `Result<Model, Error>`
    ///
    async fn create_bot_record(
        &self,
        bot_token: String,
        target_chat_id: i64,
        user_id: u64,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;

    ///
    /// Get bot record by token
    ///
    /// # Arguments
    ///
    /// * `bot_token`: bot token
    ///
    /// returns: `Result<Model, Error>` bot record model
    ///
    async fn get_bot_record_by_token(
        &self,
        bot_token: String,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model>;

    ///
    /// Check if token exists
    ///
    /// # Arguments
    ///
    /// * `bot_token`: bot token
    ///
    /// returns: `Result<bool, Error>` true if token exists
    ///
    async fn check_token_exist(&self, bot_token: String) -> anyhow::Result<bool>;

    ///
    /// Initialize bot, log out the bot and set the webhook to local api server
    ///
    /// # Arguments
    ///
    /// * `bot`: bot id
    ///
    /// returns: `Result<(), Error>`
    ///
    async fn initialize_bot(&self, bot: i64) -> anyhow::Result<()>;

    ///
    /// List bots by telegram user id
    ///
    /// # Arguments
    ///
    /// * `telegram_user_id`: telegram user id
    ///
    /// returns: `Result<Vec<Model, Global>, Error>` list of bot records
    ///
    async fn list_bots(
        &self,
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
    #[tracing::instrument(err)]
    async fn create_bot_record(
        &self,
        bot_token: String,
        target_chat_id: i64,
        user_id: u64,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model> {
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
            log::error!("Error creating bot record: {}", err);
            err
        })?;

        self.initialize_bot(bot.id).await.map_err(|err| {
            log::error!("Error initializing bot: {}", err);
            err
        })?;

        Ok(bot)
    }

    #[tracing::instrument(err)]
    async fn get_bot_record_by_token(
        &self,
        bot_token: String,
    ) -> anyhow::Result<entities::pm_forwarding_bot::Model> {
        let bot = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::BotToken.eq(bot_token))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Bot not found"))?;

        Ok(bot)
    }

    #[tracing::instrument(err)]
    async fn check_token_exist(&self, bot_token: String) -> anyhow::Result<bool> {
        let bot = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::BotToken.eq(bot_token))
            .one(&self.db)
            .await?;

        Ok(bot.is_some())
    }

    #[tracing::instrument(err)]
    async fn initialize_bot(&self, bot_id: i64) -> anyhow::Result<()> {
        let bot = entities::pm_forwarding_bot::Entity::find_by_id(bot_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Bot record not found"))?;

        let client = Bot::new(&bot.bot_token);
        client
            .log_out()
            .await
            .map_err(|err| {
                log::error!("Error logging out bot: {}", err);
            })
            .ok();

        let client = self.new_bot_client(&bot.bot_token)?;
        client
            .set_webhook(Url::parse(&format!(
                "http://pm-bot-forwarding-handler:8080/webhook/{}",
                bot.bot_token
            ))?)
            .secret_token(&bot.bot_webhook_secret)
            .await
            .map_err(|err| {
                log::error!("Error setting webhook: {}", err);
                err
            })?;

        Ok(())
    }

    #[tracing::instrument(err)]
    async fn list_bots(
        &self,
        telegram_user_id: u64,
    ) -> anyhow::Result<Vec<entities::pm_forwarding_bot::Model>> {
        let bots = entities::pm_forwarding_bot::Entity::find()
            .filter(entities::pm_forwarding_bot::Column::TelegramUserRefer.eq(telegram_user_id))
            .all(&self.db)
            .await?;

        Ok(bots)
    }
}
