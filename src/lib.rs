use teloxide::dispatching::Dispatcher;

mod handlers;
mod schema;
mod types;

pub use types::{Command, HandlerResult, MyDialogue, State};

pub async fn run() {
    dotenv::dotenv().ok();
    log4rs::init_file(dotenv::var("LOG_CONFIG").unwrap(), Default::default())
        .expect("Error initializing logger");

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
}
