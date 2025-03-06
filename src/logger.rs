pub mod logger {
    use log::LevelFilter;
    use log4rs::{
        append::{console::ConsoleAppender, file::FileAppender},
        config::{Appender, Config, Root},
        encode::pattern::PatternEncoder,
        filter::threshold::ThresholdFilter,
    };

    use crate::error::MyError;

    pub fn init_logger() -> Result<(), MyError> {
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
        Ok(())
    }
}
