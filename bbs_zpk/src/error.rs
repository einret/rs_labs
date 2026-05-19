use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AppError {
    #[error("internal error during crypto operation")]
    CryptoError { inner: bbs::errors::BBSError },
    #[error("Failed to sign entity {entity_description}, reason: {reason}")]
    SigningError {
        entity_description: String,
        reason: String,
        inner: Option<bbs::errors::BBSError>,
    },
    #[error("Invalid data: {data_name} is invalid for reason {reason}")]
    InvalidData { data_name: String, reason: String },
    #[error("Internal error: {reason}")]
    InternalError { reason: String },
}

pub(crate) type Result<T> = std::result::Result<T, AppError>;

impl From<bbs::errors::BBSError> for AppError {
    fn from(err: bbs::errors::BBSError) -> Self {
        Self::CryptoError { inner: err }
    }
}
