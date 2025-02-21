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
    /// Formats the value using the given formatter.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to use.
    ///
    /// # Returns
    ///
    /// A `Result` of the formatted string. If the inner error is also a `std::fmt::Error`,
    /// that error is forwarded. Otherwise, a new `std::fmt::Error` is created.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MyError {}

impl From<reqwest::Error> for MyError {
    /// Converts a `reqwest::Error` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `reqwest::Error` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `reqwest::Error`.
    fn from(err: reqwest::Error) -> MyError {
        MyError(Box::new(err))
    }
}

impl From<serde_json::Error> for MyError {
    /// Converts a `serde_json::Error` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `serde_json::Error` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `serde_json::Error`.
    fn from(err: serde_json::Error) -> MyError {
        MyError(Box::new(err))
    }
}

impl From<RequestError> for MyError {
    /// Converts a `RequestError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `RequestError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `RequestError`.
    fn from(err: RequestError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<InMemStorageError> for MyError {
    /// Converts an `InMemStorageError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `InMemStorageError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `InMemStorageError`.
    fn from(err: InMemStorageError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<dotenv::Error> for MyError {
    /// Converts a `dotenv::Error` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `dotenv::Error` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `dotenv::Error`.
    fn from(err: dotenv::Error) -> Self {
        MyError(Box::new(err))
    }
}

impl From<QRCodeError> for MyError {
    /// Converts a `QRCodeError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `QRCodeError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `QRCodeError`.
    fn from(err: QRCodeError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ImageError> for MyError {
    /// Converts an `ImageError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `ImageError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `ImageError`.
    fn from(err: ImageError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<io::Error> for MyError {
    /// Converts an `io::Error` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `io::Error` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `io::Error`.
    fn from(err: io::Error) -> Self {
        MyError(Box::new(err))
    }
}

impl From<&str> for MyError {
    /// Converts a `&str` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `&str` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the `&str`.
    fn from(err: &str) -> Self {
        MyError(Box::new(io::Error::new(io::ErrorKind::Other, err)))
    }
}

impl From<SetLoggerError> for MyError {
    /// Converts a `SetLoggerError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `SetLoggerError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `SetLoggerError`.
    fn from(err: SetLoggerError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ConfigError> for MyError {
    /// Converts a `ConfigError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `ConfigError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `ConfigError`.
    fn from(err: ConfigError) -> Self {
        MyError(Box::new(err))
    }
}

impl From<ConfigErrors> for MyError {
    /// Converts a `ConfigErrors` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `ConfigErrors` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `ConfigErrors`.
    fn from(err: ConfigErrors) -> Self {
        MyError(Box::new(err))
    }
}

impl From<String> for MyError {
    /// Converts a `String` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `String` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `String`.
    fn from(err: String) -> Self {
        MyError(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            err,
        )))
    }
}

impl From<ToStrError> for MyError {
    /// Converts a `ToStrError` into a `MyError`.
    ///
    /// # Arguments
    ///
    /// * `err` - The `ToStrError` to convert.
    ///
    /// # Returns
    ///
    /// A `MyError` that wraps the original `ToStrError`.
    fn from(err: ToStrError) -> Self {
        MyError(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        )))
    }
}
