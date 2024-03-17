use fast_qr::convert::Builder;
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

pub(crate) async fn qrcode_handler(bot: &Bot, message: &Message, text: &str) -> anyhow::Result<()> {
    let qr_code = match fast_qr::QRBuilder::new(text).build() {
        Ok(d) => d,
        Err(err) => {
            send_error_message!(bot, message, format!("Failed to generate QRCode: {}", err));
            return Err(anyhow::anyhow!("Failed to generate QRCode: {}", err));
        }
    };

    let image = fast_qr::convert::image::ImageBuilder::default()
        .shape(fast_qr::convert::Shape::Square)
        .background_color([255, 255, 255, 0])
        .fit_width(512)
        .to_bytes(&qr_code)?;

    bot.send_photo(
        message.chat.id,
        InputFile::memory(image).file_name("qrcode.png"),
    )
    .reply_to_message_id(message.id)
    .send()
    .await?;

    Ok(())
}
