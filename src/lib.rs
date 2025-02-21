use crate::error::MyError;
use teloxide::dispatching::Dispatcher;

pub mod error;
pub mod handlers;
pub mod schema;
pub mod types;
pub mod xui_api;

pub use types::{Command, HandlerResult, MyDialogue, State};

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

    // Initialize the bot using the environment configuration
    let bot = teloxide::Bot::from_env();

    // Set up the dispatcher with the schema
    Dispatcher::builder(bot, schema::schema())
        // Dependencies are services that can be used by the handlers
        .dependencies(dptree::deps![
            // We use an in-memory storage to store the dialogue state
            teloxide::dispatching::dialogue::InMemStorage::<State>::new()
        ])
        // Enable a control-C handler for graceful shutdown
        .enable_ctrlc_handler()
        // Build the dispatcher
        .build()
        // Start dispatching updates asynchronously
        .dispatch()
        .await;

    Ok(())
}
