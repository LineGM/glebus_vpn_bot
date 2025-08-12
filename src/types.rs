use crate::error::MyError;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступны следующие команды:")]
pub enum Command {
    #[command(description = "Показывает этот текст.")]
    Help,
    #[command(description = "Запускает операцию добавления подключений к GlebusVPN.")]
    Start,
}

pub type HandlerResult = Result<(), MyError>;
