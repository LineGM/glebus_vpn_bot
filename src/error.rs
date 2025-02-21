use fast_qr::{convert::image::ImageError, qr::QRCodeError};
use log::SetLoggerError;
use log4rs::config::runtime::{ConfigError, ConfigErrors};
use reqwest::header::ToStrError;
use std::fmt;
use std::io;
use teloxide::{dispatching::dialogue::InMemStorageError, RequestError};

#[derive(Debug)]
pub struct MyError(pub Box<dyn std::error::Error + Send + Sync + 'static>);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyError {}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> MyError {
        MyError(Box::new(err))
    }
}

impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> MyError {
        MyError(Box::new(err))
    }
}

impl From<RequestError> for MyError {
    fn from(err: RequestError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<InMemStorageError> for MyError {
    fn from(err: InMemStorageError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<dotenv::Error> for MyError {
    fn from(err: dotenv::Error) -> Self {
        MyError(Box::new(err))
    }
}

impl From<QRCodeError> for MyError {
    fn from(err: QRCodeError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ImageError> for MyError {
    fn from(err: ImageError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> Self {
        MyError(Box::new(err))
    }
}

impl From<&str> for MyError {
    fn from(err: &str) -> Self {
        MyError(Box::new(io::Error::new(io::ErrorKind::Other, err)))
    }
}

impl From<SetLoggerError> for MyError {
    fn from(err: SetLoggerError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ConfigError> for MyError {
    fn from(err: ConfigError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ConfigErrors> for MyError {
    fn from(err: ConfigErrors) -> Self {
        MyError(Box::new(err))
    }
}

impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            err,
        )))
    }
}

impl From<ToStrError> for MyError {
    fn from(err: ToStrError) -> Self {
        MyError(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        )))
    }
}
