use crate::drive_client::{DriveJobsClient, JobResponse};
use crate::face_recognize;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use ort::session::Session;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Fixed input dimensions required by the InsightFace SCRFD ONNX model.
const MODEL_W: u32 = 640;
const MODEL_H: u32 = 640;
/// Confidence threshold for keeping a detection.
const DETECT_THRESHOLD: f32 = 0.5;
/// IoU threshold for non-maximum suppression.
const NMS_THRESHOLD: f32 = 0.4;

pub struct FaceDetectDeps<'a> {
    pub drive: &'a DriveJobsClient,
    pub photos_url: &'a str,
    /// Loaded InsightFace SCRFD session (None → skip face detection).
    pub face_session: Option<Arc<Mutex<Session>>>,
    /// Loaded ArcFace recognition session (None → skip embedding generation).
    pub recognition_session: Option<Arc<Mutex<Session>>>,
    pub http: &'a reqwest::Client,
}

pub async fn process_face_detect(
    deps: FaceDetectDeps<'_>,
    job: &JobResponse,
) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        photo_id: String,
        file_id: String,
        user_id: String,
    }

    let payload: Payload = serde_json::from_value(job.payload.clone())
        .map_err(|e| format!("Invalid payload: {}", e))?;

    let session = match deps.face_session {
        Some(s) => s,
        None => {
            warn!(
                "No face detection model loaded — skipping face_detect for photo {}",
                payload.photo_id
            );
            return Ok(());
        }
    };

    // Fetch the image from drive.
    let (image_bytes, mime_type) = deps.drive.get_file_content(&payload.file_id).await?;
    if !mime_type.starts_with("image/") {
        return Err(format!("Not an image (mime_type={})", mime_type));
    }

    let photo_id = payload.photo_id.clone();
    let user_id = payload.user_id.clone();
    let photos_url = deps.photos_url.to_string();
    let http = deps.http.clone();
    let recognition_session = deps.recognition_session.clone();

    // Decode image, run inference, and crop face thumbnails on the blocking pool.
    let results = tokio::task::spawn_blocking(move || {
        detect_and_crop(&image_bytes, session, recognition_session.as_ref())
    })
    .await
    .map_err(|e| format!("Face detection task panicked: {}", e))??;

    info!("Photo {}: {} face(s) detected", photo_id, results.len());

    // POST each detected face to the photos service (failures are non-fatal).
    for (i, (bounding_box_json, thumb_b64, embedding)) in results.into_iter().enumerate() {
        let thumb_mime = thumb_b64.as_ref().map(|_| "image/jpeg");
        let body = serde_json::json!({
            "boundingBox": bounding_box_json,
            "thumbnail": thumb_b64,
            "thumbnailMimeType": thumb_mime,
            "embedding": embedding,
        });

        let save_url = format!("{}/api/v1/photos/{}/faces", photos_url, photo_id);
        match http.post(&save_url).json(&body).send().await {
            Ok(r) if r.status().is_success() => {
                info!("Saved face {} for photo {}", i, photo_id);
            }
            Ok(r) => {
                warn!(
                    "Photos service returned {} saving face {} for photo {}",
                    r.status(), i, photo_id
                );
            }
            Err(e) => {
                warn!("Failed to save face {} for photo {}: {}", i, photo_id, e);
            }
        }
    }

    // Enqueue face_cluster job for this user so clusters are recomputed.
    if let Err(e) = deps
        .drive
        .enqueue_job(
            "face_cluster",
            serde_json::json!({ "userId": user_id }),
            120,
            deps.drive.worker_secret(),
        )
        .await
    {
        warn!("Failed to enqueue face_cluster job for user {}: {}", user_id, e);
    }

    Ok(())
}

/// Decode the image, run InsightFace SCRFD detection, and return one entry per face:
/// `(bounding_box_json, Option<base64_jpeg_thumbnail>, Option<embedding_vec>)`.
fn detect_and_crop(
    image_bytes: &[u8],
    session: Arc<Mutex<Session>>,
    recognition_session: Option<&Arc<Mutex<Session>>>,
) -> Result<Vec<(serde_json::Value, Option<String>, Option<Vec<f32>>)>, String> {
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (orig_w, orig_h) = (img.width(), img.height());

    // Resize to the model's fixed 640×640 input and build the NCHW f32 tensor.
    let resized = img.resize_exact(MODEL_W, MODEL_H, image::imageops::FilterType::Lanczos3);
    let tensor = image_to_nchw_tensor(&resized)?;

    // Run inference (requires &mut Session).
    let mut faces = {
        let mut sess = session
            .lock()
            .map_err(|_| "Face session lock poisoned".to_string())?;
        insightface::detect_faces(&mut sess, tensor, DETECT_THRESHOLD)
    };
    faces = insightface::non_maximum_suppression(faces, NMS_THRESHOLD);

    // Scale factors to map model-space coords back to the original image dimensions.
    let scale_x = orig_w as f32 / MODEL_W as f32;
    let scale_y = orig_h as f32 / MODEL_H as f32;

    // Convert original image to Rgba32F once for crop_face calls.
    let rgba32f = img.into_rgba32f();

    let mut results = Vec::with_capacity(faces.len());
    for face in &faces {
        // bbox is (x1, y1, x2, y2) in model space — scale to original image space.
        let (mx1, my1, mx2, my2) = face.bbox;
        let ox1 = (mx1 * scale_x).max(0.0);
        let oy1 = (my1 * scale_y).max(0.0);
        let ox2 = (mx2 * scale_x).min(orig_w as f32);
        let oy2 = (my2 * scale_y).min(orig_h as f32);

        let bounding_box_json = serde_json::json!({
            "x": ox1,
            "y": oy1,
            "width": (ox2 - ox1).max(0.0),
            "height": (oy2 - oy1).max(0.0),
            "confidence": face.score,
            "imageWidth": orig_w,
            "imageHeight": orig_h,
        });

        // Scale keypoints to original image space, then use crop_face for
        // landmark-aligned face cropping (produces a 112×112 aligned crop).
        let scaled_kps: [(f32, f32); 5] = std::array::from_fn(|i| {
            (face.keypoints[i].0 * scale_x, face.keypoints[i].1 * scale_y)
        });
        let cropped = insightface::crop_face(&rgba32f, &scaled_kps, 112);

        // Generate ArcFace embedding if recognition model is available.
        let embedding = recognition_session.and_then(|sess| {
            match face_recognize::compute_embedding(&cropped, sess) {
                Ok(emb) => Some(emb),
                Err(e) => {
                    warn!("Failed to compute face embedding: {}", e);
                    None
                }
            }
        });

        let thumb_b64 = match rgba32f_to_jpeg_b64(&cropped) {
            Ok(b64) => Some(b64),
            Err(e) => {
                warn!("Failed to encode face crop: {}", e);
                None
            }
        };

        results.push((bounding_box_json, thumb_b64, embedding));
    }

    Ok(results)
}

/// Build a [1, 3, H, W] NCHW f32 ndarray from a DynamicImage.
/// Normalizes uint8 pixels via `(value - 127.5) / 128.0` as expected by SCRFD.
fn image_to_nchw_tensor(
    img: &DynamicImage,
) -> Result<ndarray::Array<f32, ndarray::Dim<[usize; 4]>>, String> {
    let rgb: RgbImage = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let mut array = ndarray::Array::zeros([1, 3, h, w]);
    for y in 0..h {
        for x in 0..w {
            let image::Rgb([r, g, b]) = *rgb.get_pixel(x as u32, y as u32);
            array[[0, 0, y, x]] = (r as f32 - 127.5) / 128.0;
            array[[0, 1, y, x]] = (g as f32 - 127.5) / 128.0;
            array[[0, 2, y, x]] = (b as f32 - 127.5) / 128.0;
        }
    }
    Ok(array)
}

/// Convert an `Rgba32FImage` (output of `insightface::crop_face`) to a base64 JPEG string.
fn rgba32f_to_jpeg_b64(img: &image::Rgba32FImage) -> Result<String, String> {
    let (w, h) = img.dimensions();
    let mut rgb = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels() {
        let r = (px[0].clamp(0.0, 1.0) * 255.0) as u8;
        let g = (px[1].clamp(0.0, 1.0) * 255.0) as u8;
        let b = (px[2].clamp(0.0, 1.0) * 255.0) as u8;
        rgb.put_pixel(x, y, Rgb([r, g, b]));
    }
    let mut buf = Vec::new();
    DynamicImage::ImageRgb8(rgb)
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("JPEG encode failed: {}", e))?;
    Ok(BASE64.encode(&buf))
}
