use thiserror::Error;

/// Compressed BBS+ signature size in bytes (see `bbs::prelude::Signature::to_bytes_compressed_form`).
pub type CompressedSignature = [u8; 112];

#[derive(Error, Debug)]
pub enum AppError {
    #[error("crypto error: {inner}")]
    CryptoError { inner: bbs::errors::BBSError },
    #[error("failed to sign entity {entity_description}: {reason}")]
    SigningError {
        entity_description: String,
        reason: String,
        inner: Option<bbs::errors::BBSError>,
    },
    #[error("verification failed: {reason}")]
    VerificationFailed { reason: String },
    #[allow(dead_code)]
    #[error("invalid data: {data_name} is invalid for reason {reason}")]
    InvalidData { data_name: String, reason: String },
    #[error("internal error: {reason}")]
    InternalError { reason: String },
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;

impl From<bbs::errors::BBSError> for AppError {
    fn from(err: bbs::errors::BBSError) -> Self {
        Self::CryptoError { inner: err }
    }
}
