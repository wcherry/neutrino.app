use image::Rgba32FImage;
use ort::session::Session;
use ort::value::TensorRef;
use std::sync::{Arc, Mutex};

/// Input dimensions expected by ArcFace recognition models (e.g. w600k_r50).
const REC_W: u32 = 112;
const REC_H: u32 = 112;

/// Run ArcFace recognition on an aligned face crop (output of `insightface::crop_face`).
/// Returns an L2-normalized 512-dimensional embedding vector, or an error string.
pub fn compute_embedding(
    face_crop: &Rgba32FImage,
    session: &Arc<Mutex<Session>>,
) -> Result<Vec<f32>, String> {
    // Resize to 112×112 if the crop is a different size.
    let resized: Rgba32FImage = if face_crop.width() == REC_W && face_crop.height() == REC_H {
        face_crop.clone()
    } else {
        let dyn_img = image::DynamicImage::ImageRgba32F(face_crop.clone());
        dyn_img
            .resize_exact(REC_W, REC_H, image::imageops::FilterType::Lanczos3)
            .into_rgba32f()
    };

    // Build NCHW f32 tensor [1, 3, 112, 112].
    // ArcFace normalization: (pixel / 1.0 - 0.5) / 0.5  (input pixels are already in [0,1])
    let (w, h) = (REC_W as usize, REC_H as usize);
    let mut array = ndarray::Array::zeros([1, 3, h, w]);
    for y in 0..h {
        for x in 0..w {
            let px = resized.get_pixel(x as u32, y as u32);
            // px channels are already in [0, 1]
            array[[0, 0, y, x]] = (px[0] - 0.5) / 0.5; // R
            array[[0, 1, y, x]] = (px[1] - 0.5) / 0.5; // G
            array[[0, 2, y, x]] = (px[2] - 0.5) / 0.5; // B
        }
    }

    let mut sess = session
        .lock()
        .map_err(|_| "Recognition session lock poisoned".to_string())?;

    let tensor_ref =
        TensorRef::from_array_view(array.view()).map_err(|e| format!("ort tensor error: {}", e))?;
    let inputs = ort::inputs![tensor_ref];
    let outputs = sess
        .run(inputs)
        .map_err(|e| format!("Recognition inference error: {}", e))?;

    // The first output is the embedding [1, 512].
    let output = outputs
        .values()
        .next()
        .ok_or("No outputs from recognition model")?;
    let (_shape, flat_slice) = output
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Failed to extract embedding tensor: {}", e))?;
    let flat: Vec<f32> = flat_slice.to_vec();
    if flat.is_empty() {
        return Err("Empty embedding vector".to_string());
    }

    Ok(l2_normalize(flat))
}

/// L2-normalize a vector in place and return it.
pub fn l2_normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-8 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

/// Cosine distance between two L2-normalized vectors: 1 - dot(a, b).
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    1.0 - dot.clamp(-1.0, 1.0)
}
