use super::handlers;
use crate::error::MyError;
use dptree::case;
use teloxide::{dispatching::UpdateHandler, prelude::*};

/// A root update handler for the bot.
///
/// It handles the following commands:
/// - `/help`: shows the help message
/// - `/start`: starts the VPN setup process
///
/// All other messages and callback queries are handled accordingly.
pub fn schema() -> UpdateHandler<MyError> {
    let command_handler = teloxide::filter_command::<super::Command, _>()
        .branch(case![super::Command::Help].endpoint(handlers::help))
        .branch(case![super::Command::Start].endpoint(handlers::start));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(handlers::invalid_input));

    let callback_handler = Update::filter_callback_query().endpoint(handlers::handle_callback);

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}
