//! Image storage adapters.
//!
//! `LocalServer` writes to disk under `IMAGE_STORAGE_ROOT/store_<id>/<uuid>.<ext>`
//! and is the only fully-functional adapter. S3, GCS, Cloudinary and
//! AzureBlob are stubs returning a clear "not yet wired" error so a
//! misconfigured store fails loudly.

mod azure_blob_adapter;
mod cloudinary_adapter;
mod gcs_adapter;
mod local_server_adapter;
mod registry;
mod s3_adapter;
mod storage_adapter;

pub use azure_blob_adapter::AzureBlobAdapter;
pub use cloudinary_adapter::CloudinaryAdapter;
pub use gcs_adapter::GcsAdapter;
pub use local_server_adapter::LocalServerAdapter;
pub use registry::{DefaultImageStorageRegistry, ImageStorageRegistry};
pub use s3_adapter::S3Adapter;
pub use storage_adapter::{ImageStorageAdapter, UploadRequest, UploadResult};
