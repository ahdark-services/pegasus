use teloxide::prelude::*;

use pegasus_common::bot::channel::MqUpdateListener;

use crate::handlers::{export_sticker_handler, Command};

pub(crate) async fn run(bot: Bot, listener: MqUpdateListener) {
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .branch(dptree::case![Command::ExportSticker].endpoint(export_sticker_handler)),
    );

    Dispatcher::builder(bot, handler)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
