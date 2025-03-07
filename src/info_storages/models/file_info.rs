use std::collections::HashMap;

use crate::{errors::RustusError, RustusResult};
use base64::{engine::general_purpose, Engine};
use chrono::{serde::ts_seconds, DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Information about file.
/// It has everything about stored file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub offset: usize,
    pub length: Option<usize>,
    pub path: Option<String>,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    pub deferred_size: bool,
    pub is_partial: bool,
    pub is_final: bool,
    pub parts: Option<Vec<String>>,
    pub storage: String,
    pub metadata: HashMap<String, String>,
    /// It's not a part of the protocol.
    /// Finalized sha256 hash of the file, only for business logic will be used from hook.
    /// Not tested appropriately.
    pub finalized_sha256: Option<String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub sha256: Sha256,
}

impl FileInfo {
    /// Creates new `FileInfo`.
    ///
    /// # Params
    ///
    /// File info takes
    /// `file_id` - Unique file identifier;
    /// `file_size` - Size of a file if it's known;
    /// `path` - local path of a file;
    /// `initial_metadata` - meta information, that could be omitted.
    pub fn new(
        file_id: &str,
        length: Option<usize>,
        path: Option<String>,
        storage: String,
        initial_metadata: Option<HashMap<String, String>>,
    ) -> FileInfo {
        let id = String::from(file_id);

        let mut deferred_size = true;
        if length.is_some() {
            deferred_size = false;
        }
        let metadata = match initial_metadata {
            Some(meta) => meta,
            None => HashMap::new(),
        };

        FileInfo {
            id,
            path,
            length,
            storage,
            metadata,
            deferred_size,
            offset: 0,
            is_final: false,
            is_partial: false,
            parts: None,
            created_at: chrono::Utc::now(),
            finalized_sha256: None,
            sha256: Sha256::new(),
        }
    }

    /// Function to construct `String` value
    /// from file metadata `HashMap`.
    ///
    /// This algorithm can be found at
    /// [protocol page](https://tus.io/protocols/resumable-upload.html#upload-metadata).
    pub fn get_metadata_string(&self) -> Option<String> {
        let mut result = Vec::new();

        // Getting all metadata keys.
        for (key, val) in &self.metadata {
            let encoded_value = general_purpose::STANDARD.encode(val);
            // Adding metadata entry to the list.
            result.push(format!("{key} {encoded_value}"));
        }

        if result.is_empty() {
            None
        } else {
            // Merging the metadata.
            Some(result.join(","))
        }
    }

    pub fn get_filename(&self) -> &str {
        self.metadata.get("filename").unwrap_or(&self.id)
    }

    pub async fn json(&self) -> RustusResult<String> {
        let info_clone = self.clone();
        tokio::task::spawn_blocking(move || {
            serde_json::to_string(&info_clone).map_err(RustusError::from)
        })
        .await
        .map_err(|err| {
            error!("{}", err);
            RustusError::UnableToWrite("Can't serialize info".into())
        })?
    }

    pub async fn from_json(data: String) -> RustusResult<Self> {
        tokio::task::spawn_blocking(move || {
            serde_json::from_str::<Self>(data.as_str()).map_err(RustusError::from)
        })
        .await
        .map_err(|err| {
            error!("{}", err);
            RustusError::UnableToWrite("Can't serialize info".into())
        })?
    }

    pub fn finalize_sha256(&mut self) -> RustusResult<()> {
        let hash = self.sha256.clone().finalize();
        let hex_str = hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        log::debug!("Finalized sha256: {}", hex_str);
        self.finalized_sha256 = Some(hex_str);
        Ok(())
    }

    #[cfg(test)]
    pub fn new_test() -> Self {
        FileInfo::new(
            uuid::Uuid::new_v4().to_string().as_str(),
            Some(10),
            Some("random_path".into()),
            "random_storage".into(),
            None,
        )
    }
}
