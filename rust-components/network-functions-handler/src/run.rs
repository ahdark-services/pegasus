use moka::future::Cache;
use teloxide::prelude::*;

use pegasus_common::bot::channel::MqUpdateListener;

use crate::handlers::{qrcode_handler, Command};

pub(crate) async fn run(bot: Bot, listener: MqUpdateListener) {
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .branch(dptree::case![Command::QRCode(string)].endpoint(qrcode_handler)),
    );

    let cache: Cache<String, Vec<u8>> = Cache::new(1000);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![cache])
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
