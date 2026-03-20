use exif::{In, Reader, Tag, Value};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use std::io::Cursor;

/// Maximum dimension (width or height) for image thumbnails.
const MAX_THUMB_DIM: u32 = 400;

// ── Public API ─────────────────────────────────────────────────────────────────

/// Dispatch to the appropriate thumbnail generator based on MIME type.
/// Returns JPEG-encoded thumbnail bytes.
pub fn generate_thumbnail_for_type(data: &[u8], mime_type: &str) -> Result<Vec<u8>, String> {
    let mime = mime_type.split(';').next().unwrap_or(mime_type).trim();

    if mime.starts_with("image/") {
        return generate_jpeg_thumbnail(data);
    }

    match mime {
        "application/pdf" => generate_pdf_thumbnail(data),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/msword" => generate_docx_thumbnail(data),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        | "application/vnd.ms-powerpoint" => generate_pptx_thumbnail(data),
        "text/csv" | "application/csv" | "text/comma-separated-values" => {
            generate_csv_thumbnail(data)
        }
        "application/x-neutrino-doc" => generate_neutrino_doc_thumbnail(),
        "application/x-neutrino-sheet" => generate_neutrino_sheet_thumbnail(data),
        "application/x-neutrino-slide" => generate_neutrino_slide_thumbnail(),
        _ => Err(format!("Unsupported MIME type for thumbnail: {}", mime_type)),
    }
}


/// Decode `data` as an image, apply EXIF orientation, resize to fit within
/// `MAX_THUMB_DIM`×`MAX_THUMB_DIM`, and return the result encoded as JPEG bytes.
pub fn generate_jpeg_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let img = apply_exif_orientation(img, data);

    let thumbnail = img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM);

    encode_jpeg_dynamic(thumbnail)
}

// ── Format-specific generators ─────────────────────────────────────────────────

fn generate_pdf_thumbnail(_data: &[u8]) -> Result<Vec<u8>, String> {
    // Red accent: PDF
    make_document_thumbnail([220, 38, 38])
}

fn generate_docx_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    // Try to extract the embedded Office thumbnail from the ZIP archive
    if let Some(thumb) = extract_office_thumbnail(data) {
        if let Ok(img) = image::load_from_memory(&thumb) {
            return encode_jpeg_dynamic(img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM));
        }
    }
    // Blue accent: Word
    make_document_thumbnail([37, 99, 235])
}

fn generate_pptx_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    if let Some(thumb) = extract_office_thumbnail(data) {
        if let Ok(img) = image::load_from_memory(&thumb) {
            return encode_jpeg_dynamic(img.thumbnail(MAX_THUMB_DIM, MAX_THUMB_DIM));
        }
    }
    // Orange accent: PowerPoint
    make_slide_thumbnail([217, 119, 6])
}

fn generate_csv_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data);

    let mut all_rows: Vec<Vec<String>> = Vec::new();
    for result in rdr.records().take(7) {
        match result {
            Ok(record) => all_rows.push(record.iter().map(|s| s.to_string()).collect()),
            Err(_) => break,
        }
    }

    let headers = all_rows.first().cloned().unwrap_or_default();
    let data_rows = if all_rows.len() > 1 {
        all_rows[1..].to_vec()
    } else {
        Vec::new()
    };

    // Green accent: CSV
    make_spreadsheet_thumbnail(&headers, &data_rows, [22, 163, 74])
}

fn generate_neutrino_doc_thumbnail() -> Result<Vec<u8>, String> {
    // Indigo accent: Neutrino Doc
    make_document_thumbnail([79, 70, 229])
}

fn generate_neutrino_sheet_thumbnail(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut headers: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();

    // FortuneSheet format: array of sheet objects, each with a `celldata` array.
    // Each cell: { r: row, c: col, v: { v: value } }
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(data) {
        if let Some(sheets) = json.as_array() {
            if let Some(first_sheet) = sheets.first() {
                if let Some(celldata) = first_sheet.get("celldata").and_then(|v| v.as_array()) {
                    let mut grid: std::collections::HashMap<(i64, i64), String> =
                        std::collections::HashMap::new();
                    let mut max_row = 0i64;
                    let mut max_col = 0i64;

                    for cell in celldata.iter().take(60) {
                        let r = cell.get("r").and_then(|v| v.as_i64()).unwrap_or(0);
                        let c = cell.get("c").and_then(|v| v.as_i64()).unwrap_or(0);
                        if r >= 7 || c >= 6 {
                            continue;
                        }
                        let v = cell
                            .get("v")
                            .and_then(|v| v.get("v"))
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Number(n) => n.to_string(),
                                _ => String::new(),
                            })
                            .unwrap_or_default();
                        if r > max_row {
                            max_row = r;
                        }
                        if c > max_col {
                            max_col = c;
                        }
                        grid.insert((r, c), v);
                    }

                    if max_col >= 0 {
                        for c in 0..=(max_col.min(5)) {
                            headers.push(grid.get(&(0, c)).cloned().unwrap_or_default());
                        }
                        for r in 1..=max_row.min(5) {
                            let row: Vec<String> = (0..=(max_col.min(5)))
                                .map(|c| grid.get(&(r, c)).cloned().unwrap_or_default())
                                .collect();
                            rows.push(row);
                        }
                    }
                }
            }
        }
    }

    // Teal accent: Neutrino Sheet
    make_spreadsheet_thumbnail(&headers, &rows, [13, 148, 136])
}

fn generate_neutrino_slide_thumbnail() -> Result<Vec<u8>, String> {
    // Purple accent: Neutrino Slide
    make_slide_thumbnail([124, 58, 237])
}

// ── Visual thumbnail builders ──────────────────────────────────────────────────

/// Portrait document thumbnail (280×360): accent header bar + simulated text lines.
fn make_document_thumbnail(accent: [u8; 3]) -> Result<Vec<u8>, String> {
    const W: u32 = 280;
    const H: u32 = 360;

    let mut img = RgbImage::from_pixel(W, H, Rgb([255u8, 255u8, 255u8]));

    let accent_rgb = Rgb(accent);
    let dark_accent = Rgb([
        accent[0].saturating_sub(40),
        accent[1].saturating_sub(40),
        accent[2].saturating_sub(40),
    ]);
    let gray_line = Rgb([210u8, 210u8, 210u8]);
    let border = Rgb([220u8, 220u8, 220u8]);

    // Accent header bar
    fill_rect(&mut img, 0, 0, W, 52, accent_rgb);

    // Darker bottom edge of header
    fill_rect(&mut img, 0, 48, W, 4, dark_accent);

    // White "document icon" in the header
    fill_rect(&mut img, 18, 16, 18, 22, Rgb([255u8, 255u8, 255u8]));

    // Simulated text lines in content area
    let margin = 22u32;
    let content_w = W - margin * 2;
    let line_h = 10u32;
    let line_gap = 6u32;
    let para_gap = 16u32;

    let paragraphs: &[&[f32]] = &[
        &[0.92, 0.88, 0.72],
        &[0.85, 0.94, 0.60, 0.82],
        &[0.90, 0.87, 0.75],
        &[0.78, 0.93, 0.55],
    ];

    let mut y = 72u32;
    'outer: for para_lines in paragraphs {
        for &frac in *para_lines {
            if y + line_h > H - margin {
                break 'outer;
            }
            let lw = (content_w as f32 * frac) as u32;
            fill_rect(&mut img, margin, y, lw, line_h, gray_line);
            y += line_h + line_gap;
        }
        y += para_gap;
    }

    // 1px border
    fill_rect(&mut img, 0, 0, W, 1, border);
    fill_rect(&mut img, 0, H - 1, W, 1, border);
    fill_rect(&mut img, 0, 0, 1, H, border);
    fill_rect(&mut img, W - 1, 0, 1, H, border);

    encode_jpeg(img)
}

/// Landscape slide thumbnail (400×280): dark background with accent title block and bullets.
fn make_slide_thumbnail(accent: [u8; 3]) -> Result<Vec<u8>, String> {
    const W: u32 = 400;
    const H: u32 = 280;

    let bg = Rgb([28u8, 28u8, 44u8]);
    let mut img = RgbImage::from_pixel(W, H, bg);

    let accent_rgb = Rgb(accent);
    let text_light = Rgb([200u8, 200u8, 220u8]);
    let text_dim = Rgb([100u8, 100u8, 120u8]);

    // Bottom accent bar
    fill_rect(&mut img, 0, H - 6, W, 6, accent_rgb);

    // Title area
    fill_rect(&mut img, 40, 48, 230, 20, accent_rgb);
    fill_rect(&mut img, 40, 76, 170, 12, text_light);

    // Bullet point rows
    let items: &[(u32, u32, u32)] = &[
        (40, 116, 180),
        (40, 142, 200),
        (40, 168, 155),
        (40, 194, 175),
    ];
    for &(x, y, line_w) in items {
        fill_rect(&mut img, x, y + 2, 8, 8, accent_rgb);
        fill_rect(&mut img, x + 18, y, line_w, 10, text_dim);
    }

    // Right-side image placeholder
    fill_rect(&mut img, 276, 96, 100, 100, Rgb([45u8, 45u8, 65u8]));
    // Diagonal cross in placeholder
    for i in 0..100u32 {
        let px = 276 + i;
        let py1 = 96 + i;
        let py2 = 96 + 99 - i;
        if px < W && py1 < H {
            img.put_pixel(px, py1, text_dim);
        }
        if px < W && py2 < H {
            img.put_pixel(px, py2, text_dim);
        }
    }

    encode_jpeg(img)
}

/// Landscape spreadsheet thumbnail (400×280): accent header row + grid lines.
fn make_spreadsheet_thumbnail(
    _headers: &[String],
    _data_rows: &[Vec<String>],
    accent: [u8; 3],
) -> Result<Vec<u8>, String> {
    const W: u32 = 400;
    const H: u32 = 280;
    const NUM_COLS: u32 = 5;
    const NUM_DATA_ROWS: u32 = 6;
    const TOP_BORDER: u32 = 4;
    const HEADER_H: u32 = 32;
    const ROW_H: u32 = 36;
    const LEFT_GUTTER: u32 = 36; // row-number column

    let col_w = (W - LEFT_GUTTER) / NUM_COLS;

    let mut img = RgbImage::from_pixel(W, H, Rgb([255u8, 255u8, 255u8]));

    let accent_rgb = Rgb(accent);
    let accent_light = Rgb([
        ((accent[0] as u32 + 255 * 2) / 3) as u8,
        ((accent[1] as u32 + 255 * 2) / 3) as u8,
        ((accent[2] as u32 + 255 * 2) / 3) as u8,
    ]);
    let grid_line = Rgb([220u8, 220u8, 220u8]);
    let alt_row = Rgb([248u8, 250u8, 248u8]);
    let row_num_bg = Rgb([242u8, 242u8, 242u8]);

    // Top accent border
    fill_rect(&mut img, 0, 0, W, TOP_BORDER, accent_rgb);

    // Column header row
    fill_rect(&mut img, 0, TOP_BORDER, W, HEADER_H, accent_light);

    // Row number gutter
    fill_rect(
        &mut img,
        0,
        TOP_BORDER + HEADER_H,
        LEFT_GUTTER,
        H - TOP_BORDER - HEADER_H,
        row_num_bg,
    );

    // Alternating row fills
    for r in 0..NUM_DATA_ROWS {
        let row_y = TOP_BORDER + HEADER_H + r * ROW_H;
        if row_y >= H {
            break;
        }
        if r % 2 == 1 {
            fill_rect(
                &mut img,
                LEFT_GUTTER,
                row_y,
                W - LEFT_GUTTER,
                ROW_H.min(H - row_y),
                alt_row,
            );
        }
    }

    // Horizontal grid lines
    for r in 0..=(NUM_DATA_ROWS + 1) {
        let line_y = TOP_BORDER + HEADER_H + r * ROW_H;
        if line_y >= H {
            break;
        }
        fill_rect(&mut img, 0, line_y, W, 1, grid_line);
    }

    // Header/data separator (accent)
    fill_rect(&mut img, 0, TOP_BORDER + HEADER_H - 2, W, 2, accent_rgb);

    // Vertical lines: gutter separator + column dividers
    fill_rect(&mut img, LEFT_GUTTER, TOP_BORDER, 1, H - TOP_BORDER, grid_line);
    for c in 0..=NUM_COLS {
        let col_x = LEFT_GUTTER + c * col_w;
        if col_x >= W {
            break;
        }
        fill_rect(&mut img, col_x, TOP_BORDER, 1, H - TOP_BORDER, grid_line);
    }

    encode_jpeg(img)
}

// ── Helpers ────────────────────────────────────────────────────────────────────

/// Try to extract an embedded thumbnail from a ZIP-based Office document
/// (DOCX, PPTX). Modern Office files include `docProps/thumbnail.jpeg`.
fn extract_office_thumbnail(data: &[u8]) -> Option<Vec<u8>> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let candidates = [
        "docProps/thumbnail.jpeg",
        "docProps/thumbnail.jpg",
        "docProps/thumbnail.png",
    ];
    for name in &candidates {
        if let Ok(mut entry) = archive.by_name(name) {
            use std::io::Read;
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_ok() && !buf.is_empty() {
                return Some(buf);
            }
        }
    }
    None
}

/// Fill an axis-aligned rectangle with `color`.
fn fill_rect(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32, color: Rgb<u8>) {
    let x_end = (x + w).min(img.width());
    let y_end = (y + h).min(img.height());
    for py in y..y_end {
        for px in x..x_end {
            img.put_pixel(px, py, color);
        }
    }
}

fn encode_jpeg(img: RgbImage) -> Result<Vec<u8>, String> {
    encode_jpeg_dynamic(DynamicImage::ImageRgb8(img))
}

fn encode_jpeg_dynamic(img: DynamicImage) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
    Ok(buf)
}

// ── EXIF orientation ──────────────────────────────────────────────────────────

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
        _ => img,
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
