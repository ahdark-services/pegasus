use image::guess_format;
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

    // send png
    bot.send_photo(message.chat.id, InputFile::memory(buffer))
        .reply_to_message_id(message.id)
        .send()
        .await?;

    Ok(())
}
