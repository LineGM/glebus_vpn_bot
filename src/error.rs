use log4rs::config::runtime::ConfigErrors;
use teloxide::{RequestError, dispatching::dialogue::InMemStorageError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Teloxide request error: {0}")]
    Teloxide(#[from] RequestError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Log4rs config error: {0}")]
    Log4rsConfigError(#[from] ConfigErrors),

    #[error("Dotenv error: {0}")]
    DotenvError(#[from] dotenv::Error),

    #[error("Custom error: {0}")]
    Custom(String),

    #[error("InMemStorage error: {0}")]
    InMemStorage(#[from] InMemStorageError),

    #[error("String error: {0}")]
    Str(String),

    #[error("SetLogger error: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),
}
