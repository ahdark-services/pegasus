use std::io::Cursor;

pub(crate) fn convert_webp_to_png(image_data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(image_data)?;
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(buffer)
}

pub(crate) fn convert_webm_to_gif(image_data: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
    todo!()
}
