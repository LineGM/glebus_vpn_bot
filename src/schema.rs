use super::{handlers, types::State};
use crate::error::MyError;
use dptree::case;
use teloxide::{
    dispatching::{UpdateHandler, dialogue, dialogue::InMemStorage},
    prelude::*,
};

/// A root update handler for the bot.
///
/// It handles the following commands:
/// - `/help`: shows the help message
/// - `/start`: starts the VPN setup process
///
/// All other messages and callback queries are silently ignored.
///
/// # Returns
///
/// An `UpdateHandler` that handles the specified commands and states.
pub fn schema() -> UpdateHandler<MyError> {
    // Create a command handler that handles the help, start, and cancel commands
    let command_handler = teloxide::filter_command::<super::Command, _>()
        // Handle the start command when the state is `Start`
        .branch(
            case![State::Start]
                // Handle the help command
                .branch(case![super::Command::Help].endpoint(handlers::help))
                // Handle the start command
                .branch(case![super::Command::Start].endpoint(handlers::start)),
        );

    // Create a message handler that handles the commands and the states
    let message_handler = Update::filter_message()
        // Handle the commands
        .branch(command_handler)
        // Ignore any other message
        .branch(dptree::endpoint(handlers::invalid_input));

    // Create a callback handler that handles the ReceiveDeviceInfo state
    let callback_handler = Update::filter_callback_query()
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "create_new_user")
                    .unwrap_or(false)
            })
            .endpoint(handlers::create_new_user),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "show_about_me")
                    .unwrap_or(false)
            })
            .endpoint(handlers::show_about_me),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "show_sub_link")
                    .unwrap_or(false)
            })
            .endpoint(handlers::show_sub_link),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "recreate_sub_link")
                    .unwrap_or(false)
            })
            .endpoint(handlers::recreate_sub_link),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "back_to_main_menu")
                    .unwrap_or(false)
            })
            .endpoint(handlers::back_to_main_menu),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "delete_me")
                    .unwrap_or(false)
            })
            .endpoint(handlers::delete_me),
        );

    // Create a dialogue that enters the specified states
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        // Handle messages
        .branch(message_handler)
        // Handle callback queries
        .branch(callback_handler)
}
