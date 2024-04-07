use std::fmt::Debug;
use std::sync::Arc;

use crate::handlers::{start_handler, BotState};
use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;

pub(crate) async fn run<'a, UListener>(
    bot: Bot,
    listener: UListener,
    redis_storage: Arc<RedisStorage<serializer::Json>>,
) where
    UListener: UpdateListener + 'a,
    UListener::Err: Debug,
{
    let handler = dptree::entry().branch(
        Update::filter_message()
            .enter_dialogue::<Message, RedisStorage<serializer::Json>, BotState>()
            .branch(dptree::case![BotState::Start].endpoint(start_handler)),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![redis_storage])
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
