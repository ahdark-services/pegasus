use std::borrow::Cow;

use opentelemetry::global;
use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::*;

use crate::services::forwarding_bot::{ForwardingBotService, IForwardingBotService};

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "state", content = "data")]
pub(crate) enum BotState {
    #[default]
    Start,
    ReceiveBotToken,
    ReceiveMessageTarget {
        bot_token: String,
    },
    ReceiveConfirmation {
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
        Ok(Some(BotState::ReceiveBotToken)) => {
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
    dialogue.update(BotState::ReceiveBotToken).await?;
    Ok(())
}

pub async fn receive_bot_token_handler(
    bot: Bot,
    update: Update,
    message: Message,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("receive_bot_token_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let bot_token = message.text().ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No text in message"),
        });

        anyhow::anyhow!("No text in message")
    })?;

    let bot_token_reg = regex::Regex::new(r"^[0-9]+:[a-zA-Z0-9_-]+$").unwrap();
    if !bot_token_reg.is_match(bot_token) {
        bot.send_message(
            message.chat.id,
            "Invalid bot token, please send a valid bot token",
        )
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

        return Ok(());
    }

    bot.send_message(
        message.chat.id,
        format!(
            "Received bot token: {}, please send me the target chat id",
            bot_token
        ),
    )
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    dialogue
        .update(BotState::ReceiveMessageTarget {
            bot_token: bot_token.to_string(),
        })
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to update dialogue state: {}", err)
        })?;

    Ok(())
}

pub async fn receive_message_target_handler(
    bot: Bot,
    update: Update,
    message: Message,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("receive_message_target_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let state = dialogue
        .get()
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to get dialogue state: {}", err)
        })?
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("No dialogue state"),
            });
            anyhow::anyhow!("No dialogue state")
        })?;

    let bot_token = match state {
        BotState::ReceiveMessageTarget { bot_token } => bot_token,
        _ => {
            cx.span().set_status(Status::Error {
                description: Cow::from("Unexpected dialogue state"),
            });
            return Err(anyhow::anyhow!("Unexpected dialogue state"));
        }
    };

    let target = message.text().ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No text in message"),
        });
        anyhow::anyhow!("No text in message")
    })?;

    let target_reg = regex::Regex::new(r"^-?[0-9]+$").unwrap();
    if !target_reg.is_match(target) {
        bot.send_message(
            message.chat.id,
            "Invalid target chat id, please send a valid chat id",
        )
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

        return Ok(());
    }

    let target = target.parse::<i64>().map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to parse target chat id: {}", err)
    })?;

    bot.send_message(
        message.chat.id,
        format!(
            r#"
        Confirm your bot settings:
        Bot token: <code>{}</code>
        Target chat id: <code>{}</code>
        "#,
            bot_token, target
        ),
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .reply_markup(teloxide::types::ReplyMarkup::inline_kb(vec![vec![
        teloxide::types::InlineKeyboardButton::callback("Confirm", "forward_bot_creation_confirm"),
        teloxide::types::InlineKeyboardButton::callback("Cancel", "forward_bot_creation_cancel"),
    ]]))
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    dialogue
        .update(BotState::ReceiveConfirmation { bot_token, target })
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to update dialogue state: {}", err)
        })?;

    Ok(())
}

pub async fn receive_cancel_handler(
    bot: Bot,
    update: Update,
    callback_query: CallbackQuery,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("receive_cancel_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let parent_msg = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    bot.send_message(parent_msg.chat.id, "Bot creation cancelled.")
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

    dialogue.reset().await?;

    bot.delete_message(parent_msg.chat.id, parent_msg.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to delete message: {}", err)
        })?;

    Ok(())
}

pub async fn receive_confirmation_handler(
    bot: Bot,
    update: Update,
    callback_query: CallbackQuery,
    dialogue: BotDialog,
    forwarding_bot_service: ForwardingBotService,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("receive_confirmation_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let state = dialogue
        .get()
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to get dialogue state: {}", err)
        })?
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("No dialogue state"),
            });
            anyhow::anyhow!("No dialogue state")
        })?;

    let (bot_token, target) = match state {
        BotState::ReceiveConfirmation { bot_token, target } => (bot_token, target),
        _ => {
            cx.span().set_status(Status::Error {
                description: Cow::from("Unexpected dialogue state"),
            });
            return Err(anyhow::anyhow!("Unexpected dialogue state"));
        }
    };

    let parent_msg = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    // create bot record
    let model = forwarding_bot_service
        .create_bot_record(&cx, bot_token.clone(), target)
        .await?;

    // send success message
    bot.send_message(
        parent_msg.chat.id,
        format!("Bot created successfully, id: {}", model.id),
    )
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    // reset dialogue state
    dialogue.reset().await?;

    // delete message
    bot.delete_message(parent_msg.chat.id, parent_msg.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to delete message: {}", err)
        })?;

    Ok(())
}
