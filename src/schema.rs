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
/// - `/cancel`: cancels the current VPN setup operation
///
/// It also handles the following states:
/// - `ReceiveDeviceCount`: receives the number of devices to setup
/// - `ReceiveDeviceInfo`: receives the platform for each device
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
        )
        // Handle the cancel command in any state
        .branch(case![super::Command::Cancel].endpoint(handlers::cancel));

    // Create a message handler that handles the commands and the states
    let message_handler = Update::filter_message()
        // Handle the commands
        .branch(command_handler)
        // Handle the ReceiveDeviceCount state
        .branch(case![State::ReceiveDeviceCount].endpoint(handlers::receive_device_count))
        // Handle the ReceiveDeviceInfo state
        .branch(
            case![State::ReceiveDeviceInfo {
                total_devices,
                current_device,
                applications
            }]
            .endpoint(handlers::receive_platform_selection),
        )
        // Ignore any other message
        .branch(dptree::endpoint(handlers::invalid_state));

    // Create a callback handler that handles the ReceiveDeviceInfo state
    let callback_handler = Update::filter_callback_query()
        .branch(
            case![State::ReceiveDeviceInfo {
                total_devices,
                current_device,
                applications
            }]
            .endpoint(handlers::receive_platform_selection),
        )
        // Добавляем обработку callback-запроса show_connections
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "show_connections")
                    .unwrap_or(false)
            })
            .endpoint(handlers::show_connections),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "edit_connections")
                    .unwrap_or(false)
            })
            .endpoint(handlers::edit_connections),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "back_to_main_menu")
                    .unwrap_or(false)
            })
            .endpoint(handlers::start_callback),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data.starts_with("edit_connection_"))
                    .unwrap_or(false)
            })
            .endpoint(handlers::edit_connection),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data.starts_with("change_platform_"))
                    .unwrap_or(false)
            })
            .endpoint(handlers::change_platform),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "change_current_connections")
                    .unwrap_or(false)
            })
            .endpoint(handlers::change_current_connections),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "add_connection")
                    .unwrap_or(false)
            })
            .endpoint(handlers::add_connection),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "delete_connections")
                    .unwrap_or(false)
            })
            .endpoint(handlers::delete_connections),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "back_to_edit_menu")
                    .unwrap_or(false)
            })
            .endpoint(handlers::edit_connections),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data.starts_with("delete_connection_"))
                    .unwrap_or(false)
            })
            .endpoint(handlers::delete_connection),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data.starts_with("device_count_"))
                    .unwrap_or(false)
            })
            .endpoint(handlers::receive_device_count_callback),
        );

    // Create a dialogue that enters the specified states
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        // Handle messages
        .branch(message_handler)
        // Handle callback queries
        .branch(callback_handler)
}
