use std::borrow::Cow;

use opentelemetry::global;
use opentelemetry::trace::{Status, TraceContextExt, Tracer};
use teloxide::dispatching::dialogue::{serializer, GetChatId, RedisStorage};
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardMarkup;

use crate::services::forwarding_bot::{ForwardingBotService, IForwardingBotService};

#[derive(Clone, Default, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "state", content = "data")]
pub(crate) enum BotState {
    #[default]
    Start,
    Started,
    CreationReceiveBotToken,
    CreationReceiveMessageTarget {
        bot_token: String,
    },
    CreationReceiveConfirmation {
        bot_token: String,
        target: i64,
    },
    ChooseBot,
    ChooseBotAction,
}

type BotDialog = Dialogue<BotState, RedisStorage<serializer::Json>>;

pub async fn start_handler(
    bot: Bot,
    update: Update,
    message: Message,
    bot_dialog: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("start_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    bot_dialog.reset().await.ok();

    // check command
    match message.text() {
        None => {
            cx.span().set_status(Status::Ok);
            return Ok(());
        }
        Some(command) => {
            if !command.starts_with("/pm_forwarding_bot") {
                log::debug!("Not pm forwarding bot command");
                cx.span().set_status(Status::Ok);
                return Ok(());
            }
        }
    }

    bot.send_message(
        message.chat.id,
        r#"
This bot allows you to forward messages from one chat to another.
You can manage your forwarding bots via these buttons.
"#
        .trim(),
    )
    .reply_markup(teloxide::types::ReplyMarkup::inline_kb(vec![vec![
        teloxide::types::InlineKeyboardButton::callback("Create", "forward_bot_creation"),
        teloxide::types::InlineKeyboardButton::callback("List", "forward_bot_list"),
    ]]))
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    bot_dialog.update(BotState::Started).await.map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to update dialogue state: {}", err)
    })?;

    Ok(())
}

pub async fn create_process_handler(
    bot: Bot,
    update: Update,
    callback_query: CallbackQuery,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("create_process_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let message = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    bot.send_message(message.chat.id, "Hello! Please, send me your bot token")
        .reply_markup(teloxide::types::ReplyMarkup::inline_kb(vec![vec![
            teloxide::types::InlineKeyboardButton::callback("Cancel", "forward_bot_cancel"),
        ]]))
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

    // update dialogue state
    dialogue.update(BotState::CreationReceiveBotToken).await?;
    Ok(())
}

pub async fn receive_bot_token_handler(
    bot: Bot,
    update: Update,
    message: Message,
    dialogue: BotDialog,
    forwarding_bot_service: ForwardingBotService,
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

    // check if bot token already exists
    {
        let is_exist = forwarding_bot_service
            .check_token_exist(&cx, bot_token.to_string())
            .await
            .map_err(|err| {
                cx.span().record_error(err.as_ref());
                anyhow::anyhow!("Failed to check token exist: {}", err)
            })?;

        if is_exist {
            bot.send_message(
                message.chat.id,
                "Bot token already exists, please send another bot token",
            )
            .reply_markup(teloxide::types::ReplyMarkup::inline_kb(vec![vec![
                teloxide::types::InlineKeyboardButton::callback("Cancel", "forward_bot_cancel"),
            ]]))
            .await
            .map_err(|err| {
                cx.span().record_error(&err);
                anyhow::anyhow!("Failed to send message: {}", err)
            })?;

            return Err(anyhow::anyhow!("Bot token already exists"));
        }
    }

    bot.send_message(
        message.chat.id,
        format!(
            "Received bot token: {}, please send me the target chat id",
            bot_token
        ),
    )
    .reply_markup(teloxide::types::ReplyMarkup::inline_kb(vec![vec![
        teloxide::types::InlineKeyboardButton::callback("Cancel", "forward_bot_cancel"),
    ]]))
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    // update dialogue state
    dialogue
        .update(BotState::CreationReceiveMessageTarget {
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
        BotState::CreationReceiveMessageTarget { bot_token } => bot_token,
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
        teloxide::types::InlineKeyboardButton::callback("Cancel", "forward_bot_cancel"),
    ]]))
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    dialogue
        .update(BotState::CreationReceiveConfirmation { bot_token, target })
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to update dialogue state: {}", err)
        })?;

    Ok(())
}

pub async fn cancel_handler(bot: Bot, update: Update, dialogue: BotDialog) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.clone().unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("receive_cancel_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let chat_id = update.chat_id().ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No chat id in update"),
        });
        anyhow::anyhow!("No chat id in update")
    })?;

    bot.send_message(chat_id, "Bot creation cancelled.")
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

    dialogue.reset().await?;

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

    let parent_msg = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

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
        BotState::CreationReceiveConfirmation { bot_token, target } => (bot_token, target),
        _ => {
            cx.span().set_status(Status::Error {
                description: Cow::from("Unexpected dialogue state"),
            });
            bot.send_message(parent_msg.chat.id, "Unexpected dialogue state")
                .await?;
            return Err(anyhow::anyhow!("Unexpected dialogue state"));
        }
    };

    // create bot record
    let model = match forwarding_bot_service
        .create_bot_record(&cx, bot_token.clone(), target, callback_query.from.id.0)
        .await
    {
        Ok(model) => model,
        Err(err) => {
            cx.span().record_error(err.as_ref());
            bot.send_message(parent_msg.chat.id, format!("Failed to create bot: {}", err))
                .await?;
            return Err(err);
        }
    };

    bot.answer_callback_query(callback_query.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to answer callback query: {}", err)
        })?;

    // send success message
    bot.edit_message_text(
        parent_msg.chat.id,
        parent_msg.id,
        format!("Bot created successfully, id: {}", model.id),
    )
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    // reset dialogue state
    dialogue.reset().await?;

    Ok(())
}

pub async fn list_process_handler(
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
            .span_builder("list_process_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let message = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    let bot_records = forwarding_bot_service
        .list_bots(&cx, callback_query.from.id.0)
        .await
        .map_err(|err| {
            cx.span().record_error(err.as_ref());
            anyhow::anyhow!("Failed to get all bot records: {}", err)
        })?;

    let mut inline_kb = bot_records
        .chunks(2)
        .map(|item| {
            item.iter()
                .map(|bot| {
                    teloxide::types::InlineKeyboardButton::callback(
                        format!("Bot {}", bot.id),
                        format!("forward_bot_list_bot_{}", bot.id),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    inline_kb.push(vec![teloxide::types::InlineKeyboardButton::callback(
        "Cancel",
        "forward_bot_cancel",
    )]);

    bot.answer_callback_query(callback_query.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to answer callback query: {}", err)
        })?;

    bot.edit_message_text(message.chat.id, message.id, "Choose bot to do actions")
        .reply_markup(InlineKeyboardMarkup::new(inline_kb))
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to send message: {}", err)
        })?;

    // update dialogue state
    dialogue.update(BotState::ChooseBot).await.map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to update dialogue state: {}", err)
    })?;

    Ok(())
}

pub async fn choose_bot_handler(
    bot: Bot,
    update: Update,
    callback_query: CallbackQuery,
    dialogue: BotDialog,
) -> anyhow::Result<()> {
    let tracer = global::tracer("pegasus/rust-components/pm-bot-forwarding-handler/handlers");
    let parent_cx = update.cx.unwrap_or_default();
    let ref cx = parent_cx.with_span(
        tracer
            .span_builder("choose_bot_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let message = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    let bot_id = callback_query
        .data
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("No data in callback query"),
            });
            anyhow::anyhow!("No data in callback query")
        })?
        .strip_prefix("forward_bot_list_bot_")
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("Invalid bot id"),
            });
            anyhow::anyhow!("Invalid bot id")
        })?
        .parse::<i64>()
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to parse bot id: {}", err)
        })?;

    bot.answer_callback_query(callback_query.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to answer callback query: {}", err)
        })?;

    bot.edit_message_text(
        message.chat.id,
        message.id,
        format!("Choose action for bot: {}", bot_id),
    )
    .reply_markup(InlineKeyboardMarkup::new(vec![vec![
        teloxide::types::InlineKeyboardButton::callback(
            "Reinitialize",
            format!("forward_bot_reinitialize_{}", bot_id),
        ),
        teloxide::types::InlineKeyboardButton::callback(
            "Delete",
            format!("forward_bot_delete_{}", bot_id),
        ),
    ]]))
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    // update dialogue state
    dialogue
        .update(BotState::ChooseBotAction)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to update dialogue state: {}", err)
        })?;

    Ok(())
}

pub async fn bot_reinitialize_handler(
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
            .span_builder("bot_reinitialize_handler")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .start_with_context(&tracer, &parent_cx),
    );

    let parent_msg = callback_query.message.ok_or_else(|| {
        cx.span().set_status(Status::Error {
            description: Cow::from("No message in callback query"),
        });
        anyhow::anyhow!("No message in callback query")
    })?;

    let bot_id = callback_query
        .data
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("No data in callback query"),
            });
            anyhow::anyhow!("No data in callback query")
        })?
        .strip_prefix("forward_bot_reinitialize_")
        .ok_or_else(|| {
            cx.span().set_status(Status::Error {
                description: Cow::from("Invalid bot id"),
            });
            anyhow::anyhow!("Invalid bot id")
        })?
        .parse::<i64>()
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to parse bot id: {}", err)
        })?;

    // reinitialize bot
    forwarding_bot_service
        .initialize_bot(&cx, bot_id)
        .await
        .map_err(|err| {
            cx.span().record_error(err.as_ref());
            anyhow::anyhow!("Failed to reinitialize bot: {}", err)
        })?;

    bot.answer_callback_query(callback_query.id)
        .await
        .map_err(|err| {
            cx.span().record_error(&err);
            anyhow::anyhow!("Failed to answer callback query: {}", err)
        })?;

    bot.edit_message_text(
        parent_msg.chat.id,
        parent_msg.id,
        format!("Bot reinitialized successfully, id: {}", bot_id),
    )
    .await
    .map_err(|err| {
        cx.span().record_error(&err);
        anyhow::anyhow!("Failed to send message: {}", err)
    })?;

    dialogue.reset().await?;

    Ok(())
}
