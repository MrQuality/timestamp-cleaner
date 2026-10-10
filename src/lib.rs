pub mod dependencies;
pub mod export;
pub mod media;
pub mod model;
pub mod process;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("Operation cancelled")]
    Cancelled,
    #[error("Media operation exceeded its time limit")]
    Timeout,
    #[error("{program} failed: {details}")]
    Tool { program: String, details: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}

pub type Result<T> = std::result::Result<T, Error>;
