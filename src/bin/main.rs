use glebus_vpn_bot::{error::MyError, logger, run};

/// Initializes and starts the main application.
///
/// This function sets up the logging configuration using log4rs, and then starts the GlebusVPN bot.
/// If an error occurs during the bot's execution, it logs the error message.
///
/// # Returns
///
/// A `Result` indicating success or failure of the application's execution.
#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv::dotenv().ok();

    logger::init_logger()?;

    run().await?;

    Ok(())
}
