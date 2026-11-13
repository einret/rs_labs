#[derive(Debug, thiserror::Error)]
pub enum RekorApiError {
    #[error("Rekor API error: {0}")]
    Rekor(#[from] sigstore_rekor::Error),

    #[error("Invalid log index {0}: exceeds maximum allowed protobuf int64")]
    InvalidLogIndex(u64),

    #[error("Failed to deserialize JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Unsupported or malformed entry body: {0}")]
    UnsupportedEntry(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ClientRekorError {
    #[error("Failed to build base HTTP client: {0}")]
    BuildClient(#[from] reqwest::Error),

    #[error("Failed to build Rekor client: {0}")]
    BuildRekor(#[from] sigstore_rekor::Error),
}
