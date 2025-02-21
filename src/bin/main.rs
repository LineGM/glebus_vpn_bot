use glebus_vpn_bot::run;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error running the bot: {}", e);
    }
}
