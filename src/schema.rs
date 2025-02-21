use super::{handlers, types::State};
use crate::error::MyError;
use dptree::case;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
};

pub fn schema() -> UpdateHandler<MyError> {
    let command_handler = teloxide::filter_command::<super::Command, _>()
        .branch(
            case![State::Start]
                .branch(case![super::Command::Help].endpoint(handlers::help))
                .branch(case![super::Command::Start].endpoint(handlers::start)),
        )
        .branch(case![super::Command::Cancel].endpoint(handlers::cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveDeviceCount].endpoint(handlers::receive_device_count))
        .branch(dptree::endpoint(handlers::invalid_state));

    let callback_handler = Update::filter_callback_query().branch(
        case![State::ReceiveDeviceInfo {
            total_devices,
            current_device,
            applications
        }]
        .endpoint(handlers::receive_platform_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_handler)
}
