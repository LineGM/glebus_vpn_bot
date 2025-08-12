use crate::messages::Messages;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "Информация обо мне",
            "show_about_me",
        )],
        vec![InlineKeyboardButton::callback(
            "Ссылка на подписку",
            "show_sub_link",
        )],
        vec![InlineKeyboardButton::callback(
            "Пересоздать подписку",
            "recreate_sub_link",
        )],
        vec![InlineKeyboardButton::callback(
            "Удалить подписку",
            "delete_me",
        )],
    ])
}

pub fn back_to_main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        Messages::ru().back(),
        "back_to_main_menu",
    )]])
}

pub fn new_user_confirmation() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        Messages::ru().new_user_confirmed(),
        "create_new_user",
    )]])
}
