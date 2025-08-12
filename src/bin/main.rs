use glebus_vpn_bot::{error::MyError, logger, run};

#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv::dotenv().ok();

    // Проверки переменных окружения
    dotenv::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");
    dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set");
    dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set");

    logger::init_logger()?;

    run().await?;

    Ok(())
}
