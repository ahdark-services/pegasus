use std::sync::Arc;

use teloxide::dispatching::dialogue::{serializer, RedisStorage};
use teloxide::prelude::*;

use pegasus_common::bot::channel::MqUpdateListener;

use crate::handlers::start;
use crate::state::State;

pub(crate) async fn run(
    bot: Bot,
    listener: MqUpdateListener,
    state_storage: Arc<RedisStorage<serializer::Json>>,
) {
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter(|message: Message| message.chat.is_private())
            .enter_dialogue::<Message, RedisStorage<serializer::Json>, State>()
            .branch(dptree::case![State::Start].endpoint(start)),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state_storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
