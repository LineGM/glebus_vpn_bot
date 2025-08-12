pub mod client;
pub mod error;
pub mod handlers;
pub mod keyboards;
pub mod logger;
pub mod messages;
pub mod schema;
pub mod types;

pub use error::MyError;
pub use types::{Command, HandlerResult};

use teloxide::dispatching::Dispatcher;

/// Starts the GlebusVPN bot and dispatches updates.
///
/// This function initializes the bot using the environment configuration,
/// sets up the dispatcher with the schema, and enables a control-C handler
/// for graceful shutdown. It then starts dispatching updates asynchronously.
///
/// # Returns
///
/// A `Result` indicating success or failure of the bot's execution.
///
/// # Errors
///
/// This function may return an error if the environment configuration is
/// invalid or if the bot fails to start.
pub async fn run() -> Result<(), MyError> {
    log::info!("Starting GlebusVPN bot...");

    let bot = teloxide::Bot::from_env();

    Dispatcher::builder(bot, schema::schema())
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
