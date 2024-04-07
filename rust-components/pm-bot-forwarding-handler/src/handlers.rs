use opentelemetry::global;
use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use std::borrow::Cow;
use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::*;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "state", content = "data")]
pub(crate) enum BotState {
    #[default]
    Start,
    ReceiveBotToken {
        id: i64,
    },
    ReceiveMessageTarget {
        id: i64,
        bot_token: String,
    },
    ReceiveConfirmation {
        id: i64,
        bot_token: String,
        target: i64,
    },
}

type BotDialog = Dialogue<BotState, RedisStorage<serializer::Json>>;

pub async fn start_handler(
    bot: Bot,
    update: Update,
    message: Message,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("start_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    // check command
    match message.text() {
        None => {
            cx.span().set_status(Status::Ok);
            return Ok(());
        }
        Some(command) => {
            if !command.starts_with("/new_pm_forwarding_bot") {
                log::debug!("Not pm forwarding bot command");
                cx.span().set_status(Status::Ok);
                return Ok(());
            }
        }
    }

    match dialogue.get().await {
        Ok(Some(BotState::Start)) => {} // continue
        Ok(None) => {}                  // continue
        Ok(Some(BotState::ReceiveBotToken { id: _id })) => {
            bot.send_message(
                message.chat.id,
                "You are already in the process of creating a new bot",
            )
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                anyhow::anyhow!("Failed to send message: {}", err)
            })?;

            // do not update dialogue state

            return Ok(());
        }
        Ok(Some(_)) => {
            bot.send_message(
                message.chat.id,
                "You are already in the process of creating a new bot, state reset",
            )
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                anyhow::anyhow!("Failed to send message: {}", err)
            })?;

            // reset dialogue state
            cx.span().set_status(Status::Error {
                description: Cow::from("Unexpected dialogue state"),
            });

            // reset dialogue state
            dialogue.reset().await?;

            todo!("delete bot record");

            return Err(anyhow::anyhow!("Unexpected dialogue state"));
        }
        Err(err) => {
            cx.span().record_error(&err);
            return Err(anyhow::anyhow!("Failed to get dialogue state: {}", err));
        }
    }

    bot.send_message(message.chat.id, "Hello! Please, send me your bot token")
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

    // update dialogue state
    todo!("create new bot record and update bot id");
    dialogue.update(BotState::ReceiveBotToken { id: 0 }).await?;
    Ok(())
}
