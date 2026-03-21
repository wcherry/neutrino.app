use reqwest::Client;
use serde_json::Value;

pub async fn process_index_content(
    client: &Client,
    drive_base_url: &str,
    file_id: &str,
    user_id: &str,
    worker_secret: &str,
) -> Result<(), String> {
    // Download file content
    let download_url = format!("{drive_base_url}/api/v1/storage/files/{file_id}");
    let resp = client
        .get(&download_url)
        .header("Authorization", format!("Bearer {worker_secret}"))
        .send()
        .await
        .map_err(|e| format!("Failed to download file {file_id}: {e}"))?;

    if !resp.status().is_success() {
        // Skip if file not found or inaccessible
        tracing::warn!(
            "Skipping content index for file {file_id}: HTTP {}",
            resp.status()
        );
        return Ok(());
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read file bytes for {file_id}: {e}"))?;

    let text = extract_text(&bytes, &content_type);

    if let Some(text_content) = text {
        let index_url = format!("{drive_base_url}/api/v1/drive/jobs/files/{file_id}/content-index");
        // Use user_id in the request body alongside the text content
        // The drive endpoint uses the authenticated user; for internal worker calls we
        // pass the user_id in the JSON body to allow the service to index correctly.
        let result = client
            .put(&index_url)
            .header("X-Worker-User-Id", user_id)
            .json(&serde_json::json!({
                "textContent": text_content,
                "userId": user_id
            }))
            .send()
            .await;

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Indexed content for file {file_id}");
            }
            Ok(resp) => {
                tracing::warn!(
                    "Content index PUT returned {} for file {file_id}",
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!("Failed to call content-index endpoint for {file_id}: {e}");
            }
        }
    } else {
        tracing::info!(
            "No extractable text content for file {file_id} (mime: {content_type})"
        );
    }

    Ok(())
}

fn extract_text(bytes: &[u8], content_type: &str) -> Option<String> {
    if content_type.starts_with("text/") {
        return Some(String::from_utf8_lossy(bytes).into_owned());
    }

    if content_type == "application/x-neutrino-doc"
        || content_type == "application/json"
        || content_type.contains("json")
    {
        if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
            let text = extract_text_from_json(&json);
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
    }

    None // TODO: PDF extraction, OCR for images
}

fn extract_text_from_json(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => {
            if let Some(Value::String(t)) = map.get("type") {
                if t == "text" {
                    if let Some(Value::String(text)) = map.get("text") {
                        return text.clone();
                    }
                }
            }
            // Walk content array (TipTap structure)
            if let Some(Value::Array(content)) = map.get("content") {
                return content
                    .iter()
                    .map(extract_text_from_json)
                    .collect::<Vec<_>>()
                    .join(" ");
            }
            map.values()
                .map(extract_text_from_json)
                .collect::<Vec<_>>()
                .join(" ")
        }
        Value::Array(arr) => arr
            .iter()
            .map(extract_text_from_json)
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}
