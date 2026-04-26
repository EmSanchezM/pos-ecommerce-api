//! Local filesystem image storage.
//!
//! Layout: `<root>/store_<store_id>/<uuid_v7>.<ext>`
//!
//! The `public_base_url` is what the API prepends to the relative path to
//! produce a renderable URL. The frontend can override with its own CDN
//! prefix later — the URL is just a hint.

use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::{NoContext, Timestamp, Uuid};

use super::storage_adapter::{ImageStorageAdapter, UploadRequest, UploadResult};
use crate::CatalogError;

/// Local server adapter. Two configurable knobs:
///   `root_path`        — filesystem directory the API has write access to
///   `public_base_url`  — what URL prefix to expose (matches the route mounted
///                        with `tower-http::ServeDir`)
#[derive(Debug, Clone)]
pub struct LocalServerAdapter {
    root_path: PathBuf,
    public_base_url: String,
}

impl LocalServerAdapter {
    pub fn new(root_path: impl Into<PathBuf>, public_base_url: impl Into<String>) -> Self {
        Self {
            root_path: root_path.into(),
            public_base_url: public_base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Pull both knobs from env vars, falling back to a sensible local default.
    pub fn from_env() -> Self {
        let root = std::env::var("IMAGE_STORAGE_ROOT").unwrap_or_else(|_| "./uploads".to_string());
        let base =
            std::env::var("IMAGE_STORAGE_PUBLIC_URL").unwrap_or_else(|_| "/uploads".to_string());
        Self::new(root, base)
    }

    fn extension_from(filename: &str, content_type: &str) -> &'static str {
        // Prefer extension from content-type; fall back to the original filename.
        match content_type {
            "image/jpeg" | "image/jpg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            "image/avif" => "avif",
            _ => match filename.rsplit_once('.').map(|(_, ext)| ext.to_lowercase()) {
                Some(ext)
                    if matches!(
                        ext.as_str(),
                        "jpg" | "jpeg" | "png" | "webp" | "gif" | "avif"
                    ) =>
                {
                    Box::leak(ext.into_boxed_str())
                }
                _ => "bin",
            },
        }
    }
}

impl Default for LocalServerAdapter {
    fn default() -> Self {
        Self::from_env()
    }
}

#[async_trait]
impl ImageStorageAdapter for LocalServerAdapter {
    async fn upload(&self, req: UploadRequest) -> Result<UploadResult, CatalogError> {
        let store_dir = self.root_path.join(format!("store_{}", req.store_id));
        fs::create_dir_all(&store_dir)
            .await
            .map_err(|e| CatalogError::Io(e.to_string()))?;

        let ext = Self::extension_from(&req.original_filename, &req.content_type);
        let id = Uuid::new_v7(Timestamp::now(NoContext));
        let filename = format!("{}.{}", id, ext);
        let abs_path = store_dir.join(&filename);

        let mut file = fs::File::create(&abs_path)
            .await
            .map_err(|e| CatalogError::Io(e.to_string()))?;
        file.write_all(&req.bytes)
            .await
            .map_err(|e| CatalogError::Io(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| CatalogError::Io(e.to_string()))?;

        let size = req.bytes.len() as i64;
        // storage_key stays relative so the same row can be served by any
        // host (helpful when the public_base_url is fronted by a CDN later).
        let storage_key = format!("store_{}/{}", req.store_id, filename);
        let url = format!("{}/{}", self.public_base_url, storage_key);

        Ok(UploadResult {
            url,
            storage_key,
            content_type: req.content_type,
            size_bytes: size,
        })
    }

    async fn delete(&self, storage_key: &str) -> Result<(), CatalogError> {
        let abs_path = self.root_path.join(storage_key);
        // Idempotent: deleting an already-missing file is not an error.
        match fs::remove_file(&abs_path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CatalogError::Io(e.to_string())),
        }
    }
}
