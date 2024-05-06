use std::fmt::Debug;
use std::sync::Arc;

use sea_orm::DatabaseConnection;
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;

use pegasus_common::bot::state::RedisStorage;
use pegasus_common::settings::Settings;

use crate::handlers::{
    bot_reinitialize_handler, cancel_handler, choose_bot_handler, create_process_handler,
    list_process_handler, receive_bot_token_handler, receive_confirmation_handler,
    receive_message_target_handler, start_handler, BotState,
};
use crate::services::forwarding_bot::ForwardingBotService;

pub(crate) async fn run<'a, UListener>(
    bot: Bot,
    listener: UListener,
    redis_storage: Arc<RedisStorage>,
    db: DatabaseConnection,
    settings: Settings,
) -> anyhow::Result<()>
where
    UListener: UpdateListener + 'a,
    UListener::Err: Debug,
{
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, RedisStorage, BotState>()
                .branch(
                    dptree::entry()
                        .filter(|m: Message| m.text().unwrap_or_default() == "/pm_forwarding_bot")
                        .endpoint(start_handler),
                )
                .branch(dptree::case![BotState::Start].endpoint(start_handler))
                .branch(
                    dptree::case![BotState::CreationReceiveBotToken]
                        .endpoint(receive_bot_token_handler),
                )
                .branch(
                    dptree::case![BotState::CreationReceiveMessageTarget { bot_token }]
                        .endpoint(receive_message_target_handler),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, RedisStorage, BotState>()
                .branch(
                    dptree::case![BotState::WaitingTopMenu]
                        .filter(|c: CallbackQuery| {
                            c.data.unwrap_or_default() == "forward_bot_creation"
                        })
                        .endpoint(create_process_handler),
                )
                .branch(
                    dptree::case![BotState::WaitingTopMenu]
                        .filter(|c: CallbackQuery| c.data.unwrap_or_default() == "forward_bot_list")
                        .endpoint(list_process_handler),
                )
                .branch(
                    dptree::case![BotState::ChooseBot]
                        .filter(|c: CallbackQuery| {
                            c.data
                                .unwrap_or_default()
                                .starts_with("forward_bot_list_bot_")
                        })
                        .endpoint(choose_bot_handler),
                )
                .branch(
                    dptree::case![BotState::ChooseBotAction(i64)]
                        .filter(|c: CallbackQuery| {
                            c.data.unwrap_or_default() == "forward_bot_reinitialize"
                        })
                        .endpoint(bot_reinitialize_handler),
                )
                .branch(
                    dptree::case![BotState::CreationReceiveConfirmation { bot_token, target }]
                        .filter(|c: CallbackQuery| {
                            c.data.unwrap_or_default() == "forward_bot_creation_confirm"
                        })
                        .endpoint(receive_confirmation_handler),
                )
                .branch(
                    dptree::entry()
                        .filter(|c: CallbackQuery| {
                            c.data.unwrap_or_default() == "forward_bot_cancel"
                        })
                        .endpoint(cancel_handler),
                ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            redis_storage,
            ForwardingBotService::new(db.clone(), settings.clone())
        ])
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;

    Ok(())
}
