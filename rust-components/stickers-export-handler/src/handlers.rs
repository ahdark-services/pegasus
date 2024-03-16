use image::guess_format;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{InputFile, MediaKind, MessageKind};
use teloxide::utils::command::BotCommands;

use crate::convert::{convert_webm_to_gif, convert_webp_to_png};

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

    let pending_message = bot
        .send_message(message.chat.id, "Processing...")
        .reply_to_message_id(message.id)
        .send()
        .await?;

    let media_sticker = if let MediaKind::Sticker(media_sticker) = &reply_to_message.media_kind {
        media_sticker
    } else {
        send_error_message!(bot, message, "You should reply to a sticker");
        return Ok(());
    };

    // convert sticker to png
    let file = bot.get_file(&media_sticker.sticker.file.id).send().await?;
    let mut buffer = Vec::new();
    log::debug!(
        "Downloading sticker: {}({}), chat id: {}",
        media_sticker.sticker.file.id,
        file.path,
        message.chat.id,
    );

    match bot.download_file(&file.path, &mut buffer).await {
        Ok(_) => {}
        Err(err) => {
            send_error_message!(
                bot,
                message,
                &format!("Failed to download sticker: {}", err)
            );
            return Err(err.into());
        }
    };

    let input_file = match guess_format(&buffer) {
        Ok(_) => match convert_webp_to_png(buffer) {
            Ok(buf) => InputFile::memory(buf).file_name("sticker.png"),
            Err(err) => {
                send_error_message!(bot, message, &format!("Failed to convert sticker: {}", err));
                return Err(err.into());
            }
        },
        Err(_) => match convert_webm_to_gif(buffer).await {
            Ok(buf) => InputFile::memory(buf).file_name("sticker.gif"),
            Err(err) => {
                send_error_message!(bot, message, &format!("Failed to convert sticker: {}", err));
                return Err(err.into());
            }
        },
    };

    // send png
    bot.send_document(message.chat.id, input_file)
        .reply_to_message_id(message.id)
        .send()
        .await?;

    bot.delete_message(pending_message.chat.id, pending_message.id)
        .send()
        .await?;

    Ok(())
}
