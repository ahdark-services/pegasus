use std::fmt::Debug;

use moka::future::Cache;
use teloxide::prelude::*;
use teloxide::update_listeners::UpdateListener;

use crate::handlers::{BotCommand, ping_handler, qrcode_handler};

pub(crate) async fn run<'a, B, UListener>(bot: B, listener: UListener)
where
    B: Requester + Clone + Send + Sync + 'static,
    UListener: UpdateListener + 'a,
    UListener::Err: Debug,
{
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<BotCommand>()
            .branch(dptree::case![BotCommand::QRCode(string)].endpoint(qrcode_handler))
            .branch(dptree::case![BotCommand::Ping(string)].endpoint(ping_handler)),
    );

    let cache: Cache<String, Vec<u8>> = Cache::new(1000);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![cache])
        .distribution_function(|_| None::<std::convert::Infallible>)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
