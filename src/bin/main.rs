use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use glebus_vpn_bot::run;

/// Initializes and starts the main application.
///
/// This function sets up the logging configuration using log4rs, and then starts the GlebusVPN bot. 
/// If an error occurs during the bot's execution, it logs the error message.
///
/// # Returns
///
/// A `Result` indicating success or failure of the application's execution.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let console_log_level = LevelFilter::Info;
    let file_log_level = LevelFilter::Trace;

    // Create a console appender
    let console_appender = ConsoleAppender::builder()
        .encoder(
            // The pattern encoder is used to format the log messages
            // {d} - {l} - {m}{n} means:
            // {d} is the date
            // {l} is the log level
            // {m} is the message
            // {n} is the line break
            Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")),
        )
        .build();

    // Create a file appender
    let file_appender = FileAppender::builder()
        .encoder(
            Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")),
        )
        .build("log/glebus_vpn_bot.log")?;

    // Create a config builder
    let config = Config::builder()
        // Add the console appender
        .appender(
            Appender::builder()
                .filter(
                    // If the log level is greater or equal to the level specified in the
                    // constructor, the log message is passed to the appender
                    Box::new(ThresholdFilter::new(console_log_level)),
                )
                .build("console_appender", Box::new(console_appender)),
        )
        // Add the file appender
        .appender(
            Appender::builder()
                .filter(
                    // If the log level is greater or equal to the level specified in the
                    // constructor, the log message is passed to the appender
                    Box::new(ThresholdFilter::new(file_log_level)),
                )
                .build("file_appender", Box::new(file_appender)),
        )
        // The root logger is the parent of all the loggers
        // The level specified in the constructor is the minimum level that is passed
        // to the appenders
        .build(
            Root::builder()
                .appender("console_appender")
                .appender("file_appender")
                .build(LevelFilter::Trace),
        )?;

    log4rs::init_config(config)?;

    // Run the bot
    if let Err(e) = run().await {
        log::error!("Error running the bot: {}", e);
    }
    Ok(())
}
