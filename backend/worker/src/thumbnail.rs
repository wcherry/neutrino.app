use image::ImageFormat;
use std::io::Cursor;

/// Maximum dimension (width or height) for generated thumbnails.
const MAX_THUMB_DIM: u32 = 400;

/// Decode `data` as an image, resize to fit within `MAX_THUMB_DIM`×`MAX_THUMB_DIM`,
/// and return the result encoded as JPEG bytes.
pub fn generate_jpeg_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let thumbnail = img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM);

    let mut buf = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JPEG thumbnail: {}", e))?;

    Ok(buf)
}
