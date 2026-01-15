use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("config parse error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("config write error: {0}")]
    ConfigWrite(#[from] toml::ser::Error),
    #[error("config file not found: {0}")]
    ConfigMissing(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("catalog item not found: {0}")]
    CatalogNotFound(String),
}

pub type AppResult<T> = Result<T, AppError>;
