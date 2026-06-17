//! Model download from HuggingFace (RFC-012 installation workflow).
//!
//! Provides an async function that streams `Message` values back to iced
//! via a `futures::channel::mpsc::Sender` so the UI can show live progress.
//!
//! The recommended model is `intfloat/multilingual-e5-small`:
//! - Apache 2.0 license
//! - ~93 MB total (ONNX weights + tokenizer)
//! - Supports 100+ languages including Japanese

use futures::channel::mpsc::Sender;
use futures::SinkExt as _;
use orbok_ui::state::Message;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt as _;

/// Information shown to the user before they confirm a download.
pub struct ModelSpec {
    pub hf_repo: &'static str,
    pub display_name: &'static str,
    pub license: &'static str,
    /// Approximate total size string shown in the UI.
    pub size_hint: &'static str,
    /// Files to download: (relative destination path, HuggingFace raw URL).
    pub files: &'static [(&'static str, &'static str)],
}

/// The recommended embedding model for orbok.
pub const RECOMMENDED: ModelSpec = ModelSpec {
    hf_repo: "intfloat/multilingual-e5-small",
    display_name: "multilingual-e5-small",
    license: "Apache 2.0",
    size_hint: "~93 MB",
    files: &[
        (
            "tokenizer.json",
            "https://huggingface.co/intfloat/multilingual-e5-small/resolve/main/tokenizer.json",
        ),
        (
            "onnx/model.onnx",
            "https://huggingface.co/intfloat/multilingual-e5-small/resolve/main/onnx/model.onnx",
        ),
    ],
};

/// Run the download of all model files, reporting progress via `tx`.
///
/// Called inside `tokio::spawn`; every `send` failure is silently
/// ignored (it means the UI was closed or the subscription was dropped).
pub async fn run(dest_dir: PathBuf, mut tx: Sender<Message>) {
    let files_total = RECOMMENDED.files.len() as u32;

    for (idx, (rel_path, url)) in RECOMMENDED.files.iter().enumerate() {
        let dest_file = dest_dir.join(rel_path);
        if let Some(parent) = dest_file.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                let _ = tx
                    .send(Message::DownloadFailed(format!(
                        "Cannot create {}: {e}",
                        parent.display()
                    )))
                    .await;
                return;
            }
        }

        let filename = rel_path.to_string();
        let _ = tx
            .send(Message::DownloadFileProgress {
                file: filename.clone(),
                bytes: 0,
                total: None,
                files_done: idx as u32,
                files_total,
            })
            .await;

        match download_file(url, &dest_file, &filename, idx as u32, files_total, &mut tx).await {
            Ok(()) => {}
            Err(e) => {
                // Clean up the incomplete file.
                let _ = tokio::fs::remove_file(&dest_file).await;
                let _ = tx.send(Message::DownloadFailed(e)).await;
                return;
            }
        }
    }

    let _ = tx
        .send(Message::DownloadAllComplete {
            dest_dir: dest_dir.to_string_lossy().to_string(),
        })
        .await;
}

// ── internals ─────────────────────────────────────────────────────────

async fn download_file(
    url: &str,
    dest: &std::path::Path,
    display_name: &str,
    files_done: u32,
    files_total: u32,
    tx: &mut Sender<Message>,
) -> Result<(), String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Request failed for {display_name}: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "HTTP {} for {display_name}",
            resp.status()
        ));
    }

    let total_bytes = resp.content_length();
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("Cannot create {}: {e}", dest.display()))?;

    let mut stream = resp.bytes_stream();
    use futures::StreamExt as _;

    while let Some(chunk_result) = stream.next().await {
        let chunk =
            chunk_result.map_err(|e| format!("Download error for {display_name}: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error for {display_name}: {e}"))?;
        downloaded += chunk.len() as u64;

        // Send progress every chunk (~16–64 KB); iced batches fast messages.
        let _ = tx
            .send(Message::DownloadFileProgress {
                file: display_name.to_string(),
                bytes: downloaded,
                total: total_bytes,
                files_done,
                files_total,
            })
            .await;
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error for {display_name}: {e}"))?;
    Ok(())
}
