use crate::CatalogError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageProviderType {
    LocalServer,
    S3,
    Gcs,
    Cloudinary,
    AzureBlob,
}

impl FromStr for StorageProviderType {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "local_server" | "local" | "filesystem" | "fs" => Ok(Self::LocalServer),
            "s3" | "aws_s3" => Ok(Self::S3),
            "gcs" | "google_cloud_storage" | "gcp" => Ok(Self::Gcs),
            "cloudinary" => Ok(Self::Cloudinary),
            "azure_blob" | "azure" => Ok(Self::AzureBlob),
            _ => Err(CatalogError::InvalidStorageProviderType),
        }
    }
}

impl fmt::Display for StorageProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::LocalServer => "local_server",
            Self::S3 => "s3",
            Self::Gcs => "gcs",
            Self::Cloudinary => "cloudinary",
            Self::AzureBlob => "azure_blob",
        };
        f.write_str(s)
    }
}
