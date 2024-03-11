use std::io::Cursor;

use image::ImageError;
use image::ImageFormat::Png;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MediaKind, MessageKind};
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum Command {
    #[command(description = "Export sticker", rename = "export_sticker")]
    ExportSticker,
}

macro_rules! send_error_message {
    ($bot:expr, $message:expr, $error_msg:expr) => {
        $bot.send_message($message.chat.id, $error_msg)
            .reply_to_message_id($message.id)
            .send()
            .await?;
    };
}

pub(crate) async fn export_sticker_handler(bot: Bot, message: Message) -> anyhow::Result<()> {
    let reply_to_message = if let Some(reply_to_message) = message.reply_to_message() {
        if let MessageKind::Common(reply_to_message) = &reply_to_message.kind {
            reply_to_message
        } else {
            send_error_message!(bot, message, "You should reply to a sticker");
            return Ok(());
        }
    } else {
        send_error_message!(bot, message, "You should reply to a sticker");
        return Ok(());
    };

    let media_sticker = if let MediaKind::Sticker(media_sticker) = &reply_to_message.media_kind {
        media_sticker
    } else {
        send_error_message!(bot, message, "You should reply to a sticker");
        return Ok(());
    };

    // convert sticker to png

    let file = bot.get_file(&media_sticker.sticker.file.id).send().await?;
    let mut buffer = Vec::new();
    bot.download_file(&file.path, &mut buffer).await?;

    let img = image::load_from_memory(&buffer)?;
    let output_data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ImageError> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut bytes);
        img.write_to(&mut cursor, Png)?;
        Ok(bytes)
    })
    .await??;

    // send png

    bot.send_photo(message.chat.id, InputFile::memory(output_data))
        .reply_to_message_id(message.id)
        .send()
        .await?;

    Ok(())
}
