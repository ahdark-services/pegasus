use teloxide::prelude::*;

use pegasus_common::bot::channel::MqUpdateListener;

use crate::handlers::{qrcode_handler, Command};

pub(crate) async fn run(bot: Bot, listener: MqUpdateListener) {
    let handler = dptree::entry().branch(
        Update::filter_message()
            .filter_command::<Command>()
            .endpoint(|bot: Bot, cmd: Command, msg: Message| async move {
                match cmd {
                    Command::QRCode(text) => qrcode_handler(&bot, &msg, text.as_str())
                        .await
                        .map_err(|e| {
                            log::error!("Error handling qrcode command: {}", e);
                            e
                        }),
                }
            }),
    );

    Dispatcher::builder(bot, handler)
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
