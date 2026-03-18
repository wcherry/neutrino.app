use exif::{In, Reader, Tag, Value};
use image::GenericImageView;
use std::io::Cursor;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exif: Option<ExifData>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub make: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iso: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focal_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps_latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps_longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_original: Option<String>,
}

pub fn extract_metadata(data: &[u8]) -> PhotoMetadata {
    let (width, height, format) = match image::guess_format(data) {
        Ok(fmt) => {
            let fmt_name = format_name(fmt).to_string();
            let dims = image::load_from_memory(data)
                .map(|img| img.dimensions())
                .ok();
            let (w, h) = dims.unwrap_or((0, 0));
            (
                if w > 0 { Some(w) } else { None },
                if h > 0 { Some(h) } else { None },
                Some(fmt_name),
            )
        }
        Err(_) => (None, None, None),
    };

    let exif = extract_exif(data);

    PhotoMetadata {
        width,
        height,
        format,
        exif,
    }
}

fn format_name(fmt: image::ImageFormat) -> &'static str {
    match fmt {
        image::ImageFormat::Jpeg => "jpeg",
        image::ImageFormat::Png => "png",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Bmp => "bmp",
        image::ImageFormat::Tiff => "tiff",
        image::ImageFormat::Ico => "ico",
        _ => "unknown",
    }
}

fn extract_exif(data: &[u8]) -> Option<ExifData> {
    let exif = Reader::new()
        .read_from_container(&mut Cursor::new(data))
        .ok()?;

    let make = get_ascii_field(&exif, Tag::Make);
    let model = get_ascii_field(&exif, Tag::Model);
    let datetime_original = get_ascii_field(&exif, Tag::DateTimeOriginal);
    let exposure_time = get_exposure_time(&exif);
    let f_number = get_rational_f64(&exif, Tag::FNumber);
    let iso = get_iso(&exif);
    let focal_length = get_rational_f64(&exif, Tag::FocalLength);
    let (gps_latitude, gps_longitude) = get_gps(&exif);

    if make.is_none()
        && model.is_none()
        && datetime_original.is_none()
        && exposure_time.is_none()
        && f_number.is_none()
        && iso.is_none()
        && focal_length.is_none()
        && gps_latitude.is_none()
    {
        return None;
    }

    Some(ExifData {
        make,
        model,
        exposure_time,
        f_number,
        iso,
        focal_length,
        gps_latitude,
        gps_longitude,
        datetime_original,
    })
}

fn get_ascii_field(exif: &exif::Exif, tag: Tag) -> Option<String> {
    exif.get_field(tag, In::PRIMARY).and_then(|f| {
        if let Value::Ascii(ref v) = f.value {
            v.first()
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .map(|s| s.trim_end_matches('\0').trim().to_string())
                .filter(|s| !s.is_empty())
        } else {
            None
        }
    })
}

fn get_rational_f64(exif: &exif::Exif, tag: Tag) -> Option<f64> {
    exif.get_field(tag, In::PRIMARY).and_then(|f| {
        if let Value::Rational(ref v) = f.value {
            v.first().map(|r| {
                if r.denom == 0 {
                    0.0
                } else {
                    r.num as f64 / r.denom as f64
                }
            })
        } else {
            None
        }
    })
}

fn get_exposure_time(exif: &exif::Exif) -> Option<String> {
    exif.get_field(Tag::ExposureTime, In::PRIMARY).and_then(|f| {
        if let Value::Rational(ref v) = f.value {
            v.first().map(|r| {
                if r.denom == 0 {
                    return "0".to_string();
                }
                if r.num == 1 {
                    format!("1/{}", r.denom)
                } else {
                    format!("{}/{}", r.num, r.denom)
                }
            })
        } else {
            None
        }
    })
}

fn get_iso(exif: &exif::Exif) -> Option<u32> {
    exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
        .and_then(|f| {
            if let Value::Short(ref v) = f.value {
                v.first().copied().map(|s| s as u32)
            } else {
                None
            }
        })
}

fn get_gps(exif: &exif::Exif) -> (Option<f64>, Option<f64>) {
    let lat = parse_gps_coord(
        exif.get_field(Tag::GPSLatitude, In::PRIMARY),
        exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY),
    );
    let lon = parse_gps_coord(
        exif.get_field(Tag::GPSLongitude, In::PRIMARY),
        exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY),
    );
    (lat, lon)
}

fn parse_gps_coord(
    coord_field: Option<&exif::Field>,
    ref_field: Option<&exif::Field>,
) -> Option<f64> {
    let field = coord_field?;
    if let Value::Rational(ref v) = field.value {
        if v.len() < 3 {
            return None;
        }
        let to_f64 = |r: &exif::Rational| {
            if r.denom == 0 {
                0.0
            } else {
                r.num as f64 / r.denom as f64
            }
        };
        let degrees = to_f64(&v[0]);
        let minutes = to_f64(&v[1]);
        let seconds = to_f64(&v[2]);
        let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

        if let Some(ref_f) = ref_field {
            if let Value::Ascii(ref rv) = ref_f.value {
                if let Some(bytes) = rv.first() {
                    if bytes.first() == Some(&b'S') || bytes.first() == Some(&b'W') {
                        decimal = -decimal;
                    }
                }
            }
        }
        Some(decimal)
    } else {
        None
    }
}
