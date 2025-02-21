use crate::error::MyError;
use log::LevelFilter;
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use teloxide::dispatching::Dispatcher;

pub mod error;
pub mod handlers;
pub mod schema;
pub mod types;
pub mod xui_api;

pub use types::{Command, HandlerResult, MyDialogue, State};

pub async fn run() -> Result<(), MyError> {
    dotenv::dotenv().ok();

    let console_log_level = LevelFilter::Info;
    let file_log_level = LevelFilter::Trace;

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build();

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build("log/glebus_vpn_bot.log")?;

    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_log_level)))
                .build("console_appender", Box::new(console_appender)),
        )
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(file_log_level)))
                .build("file_appender", Box::new(file_appender)),
        )
        .build(
            Root::builder()
                .appender("console_appender")
                .appender("file_appender")
                .build(LevelFilter::Trace),
        )?;

    log4rs::init_config(config)?;

    log::info!("Starting GlebusVPN bot...");

    let bot = teloxide::Bot::from_env();

    Dispatcher::builder(bot, schema::schema())
        .dependencies(dptree::deps![
            teloxide::dispatching::dialogue::InMemStorage::<State>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
