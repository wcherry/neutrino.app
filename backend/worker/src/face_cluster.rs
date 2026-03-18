use crate::drive_client::JobResponse;
use crate::face_recognize::cosine_distance;
use tracing::{info, warn};

pub struct FaceClusterDeps<'a> {
    pub photos_url: &'a str,
    pub http: &'a reqwest::Client,
    pub eps: f32,
    pub min_samples: usize,
}

pub async fn process_face_cluster(
    deps: FaceClusterDeps<'_>,
    job: &JobResponse,
) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        user_id: String,
    }

    let payload: Payload = serde_json::from_value(job.payload.clone())
        .map_err(|e| format!("Invalid face_cluster payload: {}", e))?;

    // Fetch all face embeddings for this user.
    let embeddings_url = format!(
        "{}/api/v1/internal/users/{}/face-embeddings",
        deps.photos_url, payload.user_id
    );

    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct FaceEntry {
        face_id: String,
        embedding: Vec<f32>,
        thumbnail: Option<String>,
        thumbnail_mime_type: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct EmbeddingsResp {
        faces: Vec<FaceEntry>,
    }

    let resp = deps
        .http
        .get(&embeddings_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch embeddings: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Photos service returned {} fetching embeddings",
            resp.status()
        ));
    }

    let data: EmbeddingsResp = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse embeddings response: {}", e))?;

    let faces = data.faces;
    if faces.is_empty() {
        info!("No face embeddings for user {} — skipping clustering", payload.user_id);
        return Ok(());
    }

    info!(
        "Clustering {} faces for user {}",
        faces.len(),
        payload.user_id
    );

    let cluster_assignments = dbscan(&faces.iter().map(|f| f.embedding.as_slice()).collect::<Vec<_>>(), deps.eps, deps.min_samples);

    // Build cluster groups: noise points (label -1) get their own single-face cluster.
    let num_clusters = cluster_assignments.iter().filter(|&&c| c >= 0).max().map(|&m| m + 1).unwrap_or(0) as usize;
    let mut clusters: Vec<Vec<usize>> = vec![vec![]; num_clusters];
    let mut noise_singles: Vec<usize> = vec![];

    for (i, &label) in cluster_assignments.iter().enumerate() {
        if label >= 0 {
            clusters[label as usize].push(i);
        } else {
            noise_singles.push(i);
        }
    }

    // Noise points become single-face clusters.
    for i in noise_singles {
        clusters.push(vec![i]);
    }

    info!(
        "Formed {} clusters for user {}",
        clusters.len(),
        payload.user_id
    );

    // Build the request body.
    let cluster_entries: Vec<serde_json::Value> = clusters
        .into_iter()
        .filter(|c| !c.is_empty())
        .map(|member_indices| {
            let face_ids: Vec<&str> = member_indices.iter().map(|&i| faces[i].face_id.as_str()).collect();
            let cover = &faces[member_indices[0]];
            serde_json::json!({
                "faceIds": face_ids,
                "coverFaceId": cover.face_id,
                "coverThumbnail": cover.thumbnail,
                "coverThumbnailMimeType": cover.thumbnail_mime_type,
            })
        })
        .collect();

    let cluster_count = cluster_entries.len();
    let body = serde_json::json!({
        "userId": payload.user_id,
        "clusters": cluster_entries,
    });

    let save_url = format!("{}/api/v1/internal/persons/clusters", deps.photos_url);
    match deps.http.post(&save_url).json(&body).send().await {
        Ok(r) if r.status().is_success() => {
            info!("Saved {} clusters for user {}", cluster_count, payload.user_id);
        }
        Ok(r) => {
            warn!(
                "Photos service returned {} saving clusters for user {}",
                r.status(),
                payload.user_id
            );
        }
        Err(e) => {
            warn!("Failed to save clusters for user {}: {}", payload.user_id, e);
        }
    }

    Ok(())
}

/// DBSCAN clustering over pre-computed L2-normalized embeddings using cosine distance.
/// Returns a label per point: ≥0 = cluster id, -1 = noise.
fn dbscan(embeddings: &[&[f32]], eps: f32, min_samples: usize) -> Vec<i32> {
    let n = embeddings.len();
    let mut labels = vec![-1i32; n];
    let mut cluster_id = 0i32;

    for i in 0..n {
        if labels[i] != -1 {
            continue; // already processed
        }
        let neighbours = region_query(embeddings, i, eps);
        if neighbours.len() < min_samples {
            // Noise — will be reassigned to a singleton cluster after the main loop.
            continue;
        }
        // Start a new cluster.
        labels[i] = cluster_id;
        let mut seed_set: Vec<usize> = neighbours;
        let mut si = 0;
        while si < seed_set.len() {
            let q = seed_set[si];
            si += 1;
            if labels[q] == -1 {
                labels[q] = cluster_id;
                let q_neighbours = region_query(embeddings, q, eps);
                if q_neighbours.len() >= min_samples {
                    for nb in q_neighbours {
                        if labels[nb] == -1 {
                            seed_set.push(nb);
                        }
                    }
                }
            } else if labels[q] < 0 {
                // Was noise, absorb into cluster.
                labels[q] = cluster_id;
            }
        }
        cluster_id += 1;
    }

    labels
}

fn region_query(embeddings: &[&[f32]], point: usize, eps: f32) -> Vec<usize> {
    embeddings
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            if cosine_distance(embeddings[point], e) <= eps {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}
