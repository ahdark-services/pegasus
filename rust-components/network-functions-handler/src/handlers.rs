use std::sync::Arc;

use fast_qr::convert::Builder;
use moka::future::Cache;
use teloxide::prelude::*;
use teloxide::types::InputFile;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum Command {
    #[command(description = "Generate QRCode", rename = "qrcode")]
    QRCode(String),
}

macro_rules! send_error_message {
    ($bot:expr, $message:expr, $error_msg:expr) => {
        $bot.send_message($message.chat.id, $error_msg)
            .reply_to_message_id($message.id)
            .send()
            .await?;
    };
}

macro_rules! match_error {
    ($data:expr, $bot:expr, $message:expr, $error_msg:expr) => {
        match $data {
            Ok(data) => data,
            Err(err) => {
                send_error_message!($bot, $message, format!($error_msg, err));
                return Err(anyhow::anyhow!($error_msg, err));
            }
        }
    };
}

pub(crate) async fn qrcode_handler(
    bot: Arc<Bot>,
    message: Message,
    text: String,
    cache: Cache<String, Vec<u8>>,
) -> anyhow::Result<()> {
    if text.is_empty() {
        send_error_message!(bot, message, "Text is empty");
        return Err(anyhow::anyhow!("Text is empty"));
    }

    // Check if QRCode already exists in cache
    if let Some(data) = cache.get(&format!("qrcode:{}", &text).to_string()).await {
        log::debug!("QRCode cache hit");

        bot.send_photo(
            message.chat.id,
            InputFile::memory(data.to_vec()).file_name("qrcode.png"),
        )
        .reply_to_message_id(message.id)
        .send()
        .await?;

        return Ok(());
    }

    let qr_code = match_error!(
        fast_qr::QRBuilder::new(text.clone()).build(),
        bot,
        message,
        "Failed to build QRCode: {}"
    );

    let image = match_error!(
        fast_qr::convert::image::ImageBuilder::default()
            .shape(fast_qr::convert::Shape::Square)
            .background_color([255, 255, 255, 0])
            .fit_width(512)
            .to_bytes(&qr_code),
        bot,
        message,
        "Failed to convert QRCode to image: {}"
    );

    bot.send_photo(
        message.chat.id,
        InputFile::memory(image.clone()).file_name("qrcode.png"),
    )
    .reply_to_message_id(message.id)
    .send()
    .await?;

    cache
        .insert(format!("qrcode:{}", text).to_string(), image.to_vec())
        .await;

    Ok(())
}
