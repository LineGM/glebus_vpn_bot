use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, utils::command::BotCommands};

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveDeviceCount,
    ReceiveDeviceInfo {
        total_devices: u8,
        current_device: u8,
        applications: Vec<String>,
    },
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Доступны следующие команды:"
)]
pub enum Command {
    #[command(description = "Показывает этот текст.")]
    Help,
    #[command(description = "Запускает операцию добавления подключений к GlebusVPN.")]
    Start,
    #[command(description = "Прерывает текущую операцию.")]
    Cancel,
}

pub type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
