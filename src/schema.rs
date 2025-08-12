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
                    .map(|data| data == "delete_me")
                    .unwrap_or(false)
            })
            .endpoint(handlers::delete_me),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data
                    .as_ref()
                    .map(|data| data == "back_to_main_menu")
                    .unwrap_or(false)
            })
            .endpoint(handlers::back_to_main_menu),
        );

    dptree::entry()
        .branch(message_handler)
        .branch(callback_handler)
}
