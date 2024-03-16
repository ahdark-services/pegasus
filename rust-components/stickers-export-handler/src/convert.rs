use std::io::Cursor;

use image::codecs::{png, webp};
use image::ImageFormat;

pub(crate) fn convert_webp_to_png(input_buffer: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let input_image = image::load_from_memory(&input_buffer)
        .map_err(|err| anyhow::anyhow!("Failed to load image from memory: {}", err))?;

    let mut output_buffer = Vec::new();
    let mut output_cursor = Cursor::new(&mut output_buffer);
    input_image
        .write_to(&mut output_cursor, ImageFormat::Png)
        .map_err(|err| anyhow::anyhow!("Failed to write image to buffer: {}", err))?;

    Ok(output_buffer)
}

pub(crate) async fn convert_webm_to_gif(input_buffer: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut input_file = async_tempfile::TempFile::new_with_name("input.webm")
        .await
        .map_err(|err| anyhow::anyhow!("Failed to create input file: {}", err))?;

    let palette_file = async_tempfile::TempFile::new_with_name("palette.png")
        .await
        .map_err(|err| anyhow::anyhow!("Failed to create palette file: {}", err))?;

    let mut output_file = async_tempfile::TempFile::new_with_name("output.gif")
        .await
        .map_err(|err| anyhow::anyhow!("Failed to create output file: {}", err))?;

    log::debug!(
        "input_file: {}, palette_file: {}, output_file: {}",
        input_file.file_path().to_str().unwrap(),
        palette_file.file_path().to_str().unwrap(),
        output_file.file_path().to_str().unwrap()
    );

    tokio::io::AsyncWriteExt::write_all(&mut input_file, &input_buffer)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to write input file: {}", err))?;

    // create a palette from the input file
    let out = tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file.file_path())
        .arg("-vf")
        .arg("palettegen")
        .arg(palette_file.file_path())
        .arg("-y")
        .output()
        .await
        .map_err(|err| anyhow::anyhow!("Failed to generate palette: {}", err))?;
    if !out.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to generate palette: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    // convert the input file to a gif using the palette
    let out = tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file.file_path())
        .arg("-i")
        .arg(palette_file.file_path())
        .arg("-filter_complex")
        .arg("paletteuse")
        .arg(output_file.file_path())
        .arg("-y")
        .output()
        .await
        .map_err(|err| anyhow::anyhow!("Failed to convert webm to gif: {}", err))?;
    if !out.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to convert webm to gif: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let mut output_buffer = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut output_file, &mut output_buffer)
        .await
        .map_err(|err| anyhow::anyhow!("Failed to read output file: {}", err))?;

    if output_buffer.is_empty() {
        return Err(anyhow::anyhow!("Empty output buffer"));
    }

    Ok(output_buffer)
}
