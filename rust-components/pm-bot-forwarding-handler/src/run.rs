use std::fmt::Debug;
use std::sync::Arc;

use sea_orm::DatabaseConnection;
use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;

use crate::handlers::{
    receive_bot_token_handler, receive_cancel_handler, receive_confirmation_handler,
    receive_message_target_handler, start_handler, BotState,
};
use crate::services::forwarding_bot::ForwardingBotService;

pub(crate) async fn run<'a, UListener>(
    bot: Bot,
    listener: UListener,
    redis_storage: Arc<RedisStorage<serializer::Json>>,
    db: DatabaseConnection,
) where
    UListener: UpdateListener + 'a,
    UListener::Err: Debug,
{
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, RedisStorage<serializer::Json>, BotState>()
                .branch(dptree::case![BotState::Start].endpoint(start_handler))
                .branch(
                    dptree::case![BotState::ReceiveBotToken].endpoint(receive_bot_token_handler),
                )
                .branch(
                    dptree::case![BotState::ReceiveMessageTarget { bot_token }]
                        .endpoint(receive_message_target_handler),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, RedisStorage<serializer::Json>, BotState>()
                .branch(
                    dptree::case![BotState::ReceiveConfirmation { bot_token, target }]
                        .filter(|u| {
                            if let CallbackQuery {
                                data: Some(data), ..
                            } = u
                            {
                                data == "forward_bot_creation_confirm"
                            } else {
                                false
                            }
                        })
                        .endpoint(receive_confirmation_handler),
                )
                .branch(
                    dptree::case![BotState::ReceiveConfirmation { bot_token, target }]
                        .filter(|u| {
                            if let CallbackQuery {
                                data: Some(data), ..
                            } = u
                            {
                                data == "forward_bot_creation_cancel"
                            } else {
                                false
                            }
                        })
                        .endpoint(receive_cancel_handler),
                ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            redis_storage,
            ForwardingBotService::new(db.clone())
        ])
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
