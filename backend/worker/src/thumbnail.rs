use exif::{In, Reader, Tag, Value};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

/// Maximum dimension (width or height) for generated thumbnails.
const MAX_THUMB_DIM: u32 = 400;

/// Decode `data` as an image, apply EXIF orientation, resize to fit within
/// `MAX_THUMB_DIM`×`MAX_THUMB_DIM`, and return the result encoded as JPEG bytes.
pub fn generate_jpeg_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let img = apply_exif_orientation(img, data);

    let thumbnail = img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM);

    let mut buf = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JPEG thumbnail: {}", e))?;

    Ok(buf)
}

/// Read the EXIF Orientation tag from `data` and apply the corresponding
/// rotation/flip to `img`.  Returns the (possibly transformed) image.
/// If the orientation tag is absent or unreadable, the image is returned as-is.
fn apply_exif_orientation(img: DynamicImage, data: &[u8]) -> DynamicImage {
    let orientation = read_exif_orientation(data).unwrap_or(1);
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img, // 1 = normal, or unknown
    }
}

fn read_exif_orientation(data: &[u8]) -> Option<u32> {
    let exif = Reader::new()
        .read_from_container(&mut Cursor::new(data))
        .ok()?;
    let field = exif.get_field(Tag::Orientation, In::PRIMARY)?;
    if let Value::Short(ref v) = field.value {
        v.first().map(|&o| o as u32)
    } else {
        None
    }
}
