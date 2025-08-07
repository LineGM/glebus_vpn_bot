use super::messages::Messages;
use super::types::{Command, HandlerResult, MyDialogue};
use chrono::{DateTime, TimeZone, Utc};
use remnawave::CreateUserRequestDto;
use remnawave::RemnawaveApiClient;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

/// Extracts the user id from a `Message` or returns "unknown" if there is none.
///
/// # Arguments
///
/// * `msg` - The `Message` to extract the user id from.
///
/// # Returns
///
/// The user id if one exists, otherwise "unknown".
fn get_user_id(msg: &Message) -> String {
    match msg.from {
        Some(ref user) => user.id.to_string(),
        None => "unknown".to_string(),
    }
}

async fn send_main_menu(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    let keyboard = InlineKeyboardMarkup::new(vec![
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
    ]);
    bot.send_message(chat_id, "Главное меню:")
        .reply_markup(keyboard)
        .await?;
    Ok(())
}

/// Handles the `/start` command.
///
/// This function logs the user's input, sends a welcome message, and updates the dialogue state to
/// `ReceiveDeviceCount`.
///
/// # Arguments
///
/// * `bot` - The bot handle.
/// * `dialogue` - The dialogue handle.
/// * `msg` - The received `Message`.
///
/// # Returns
///
/// A `HandlerResult`.
pub async fn start(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let user_id = get_user_id(&msg);

    log::info!("User {} called /start", user_id);

    let client = RemnawaveApiClient::new(
        dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
        Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
    )
    .expect("Failed to create RemnawaveApiClient");

    match client.users.get_user_by_telegram_id(user_id).await {
        Ok(_user) => {
            send_main_menu(&bot, msg.chat.id).await?;
        }
        Err(_) => {
            let keyboard = InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                Messages::ru().new_user_confirmed(),
                "create_new_user",
            )]]);
            bot.send_message(msg.chat.id, Messages::ru().welcome_prompt())
                .reply_markup(keyboard)
                .await?;
        }
    };
    Ok(())
}

/// Handles the `/help` command by sending a list of available commands to the user.
///
/// This function extracts the username and chat ID from the received message, logs
/// the `/help` command usage, and sends a message to the user with the descriptions
/// of all available commands.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages.
/// * `msg` - The received `Message` from which user information is extracted.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the operation.
pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let user_id = get_user_id(&msg); // Extract user ID from the message

    // Log the usage of the /help command with the user ID
    log::info!("User {} called /help", user_id);

    // Send a message with the descriptions of all available commands to the user
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;

    Ok(())
}

/// Handles an invalid state by sending a message to the user and logging the
/// incorrect user input.
///
/// # Arguments
///
/// * `bot` - The bot handle.
/// * `msg` - The received `Message`.
///
/// # Returns
///
/// A `HandlerResult`.
pub async fn invalid_input(bot: Bot, msg: Message) -> HandlerResult {
    // Extract the user ID from the message
    let user_id = get_user_id(&msg);
    // Extract the chat ID from the message
    let chat_id = msg.chat.id;
    // Extract the user's input from the message
    let user_input = msg.text().unwrap_or_default();

    // Log the incorrect user input
    log::warn!(
        "User {} entered an incorrect value: {}",
        user_id,
        user_input
    );

    // Send a message to the user indicating that the input was incorrect
    bot.send_message(chat_id, Messages::ru().invalid_input())
        .await?;
    Ok(())
}

pub async fn create_new_user(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id.to_string();
        let chat_id = msg.chat().id;

        log::info!("User {} called create_new_user", user_id);

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        let new_user = CreateUserRequestDto {
            username: q.from.username.clone().unwrap_or(q.from.id.to_string()),
            status: remnawave::api::types::common::UserStatus::Active,
            short_uuid: None,
            trojan_password: None,
            vless_uuid: None,
            ss_password: None,
            traffic_limit_bytes: None,
            traffic_limit_strategy: remnawave::api::types::common::TrafficLimitStrategy::NoReset,
            expire_at: Utc.with_ymd_and_hms(2099, 01, 01, 0, 0, 0).unwrap(),
            created_at: None,
            last_traffic_reset_at: None,
            description: None,
            tag: Some("USER".to_string()),
            telegram_id: Some(user_id.parse().unwrap()),
            email: None,
            hwid_device_limit: None,
            active_internal_squads: Some(
                ["9236f04f-4d48-4bd2-a24e-60a352b9897a".to_string()].to_vec(),
            ),
        };

        match client.users.create_user(new_user).await {
            Ok(_) => {
                log::info!("User {} created successfully", user_id);
                send_main_menu(&bot, chat_id).await?;
            }
            Err(e) => {
                log::error!("Failed to create user: {}", e);
                bot.send_message(chat_id, Messages::ru().error("создании пользователя"))
                    .await?;
            }
        };
    }

    Ok(())
}

pub async fn show_about_me(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id.to_string();
        let chat_id = msg.chat().id;

        log::info!("User {} called show_about_me", user_id);

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        match client.users.get_user_by_telegram_id(user_id).await {
            Ok(user) => {
                let user_data = &user.response[0];

                let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                    Messages::ru().back(),
                    "back_to_main_menu",
                )]]);

                bot.send_message(chat_id, format!("🔑 *Профиль пользователя*\n Имя пользователя: `{}`\n Статус: `{}`\n📲 *Идентификаторы*\n Telegram ID: `{}`\n Email: `{}`\n📊 *Трафик*\n Использовано за все время: `{}`\n Лимит трафика: `{}`\n🖥 *Подключения и агенты*\n Последний UserAgent: `{}`\n Первое подключение: `{}`\n⏰ *Срок действия подписки*\n Активно до: `{}`\n📥 *Ссылки*\n Подписка: `{}`\n HAPP Crypto Link: `{}`",
                    user_data.username,
                    user_data.status,
                    user_data.telegram_id.unwrap_or(0),
                    user_data.email.clone().unwrap_or("null".to_string()),
                    user_data.lifetime_used_traffic_bytes,
                    user_data.traffic_limit_bytes,
                    user_data.sub_last_user_agent.clone().unwrap_or("null".to_string()),
                    user_data.first_connected_at.unwrap_or_default(),
                    user_data.expire_at,
                    user_data.subscription_url,
                    user_data.happ.crypto_link
                ))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .reply_markup(keyboard)
                    .await?;
            }
            Err(e) => {
                log::error!("Failed to get client info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .await?;
            }
        };
    }

    Ok(())
}

pub async fn show_sub_link(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id.to_string();
        let chat_id = msg.chat().id;

        log::info!("User {} called show_sub_link", user_id);

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        match client.users.get_user_by_telegram_id(user_id).await {
            Ok(user) => {
                let user_data = &user.response[0];

                let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                    Messages::ru().back(),
                    "back_to_main_menu",
                )]]);

                bot.send_message(
                    chat_id,
                    format!("Ссылка на вашу подписку `{}`", user_data.subscription_url),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboard)
                .await?;
            }
            Err(e) => {
                log::error!("Failed to get client info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .await?;
            }
        };
    }

    Ok(())
}

pub async fn delete_me(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id.to_string();
        let chat_id = msg.chat().id;

        log::info!("User {} called delete_me", user_id);

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        match client.users.get_user_by_telegram_id(user_id.clone()).await {
            Ok(user) => {
                let user_data = &user.response[0];

                let user_uuid = user_data.uuid.clone();

                match client.users.delete_user(user_uuid).await {
                    Ok(_) => {
                        log::info!("User {} deleted successfully", user_id);
                        bot.send_message(chat_id,format!("Ваша подписка успешно удалена, для создания новой подписки используйте команду /start"))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .await?;
                    }
                    Err(e) => {
                        log::error!("Failed to delete user {}: {}", user_id, e);
                        bot.send_message(chat_id, Messages::ru().error("удалении пользователя"))
                            .await?;
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get client info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .await?;
            }
        };
    }

    Ok(())
}

pub async fn recreate_sub_link(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id.to_string();
        let chat_id = msg.chat().id;

        log::info!("User {} called recreate_sub_link", user_id);

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        match client.users.get_user_by_telegram_id(user_id.clone()).await {
            Ok(user) => {
                let user_data = &user.response[0];

                let user_uuid = user_data.uuid.clone();

                match client.users.delete_user(user_uuid).await {
                    Ok(_) => {
                        log::info!("User {} deleted successfully (during recreation)", user_id);
                        let squads: Vec<String> = user_data
                            .active_internal_squads
                            .iter()
                            .map(|s| s.clone().uuid.to_string())
                            .collect();
                        let new_user = CreateUserRequestDto {
                            username: user_data.username.clone(),
                            status: user_data.status.clone(),
                            short_uuid: None,
                            trojan_password: None,
                            vless_uuid: None,
                            ss_password: None,
                            traffic_limit_bytes: Some(0),
                            traffic_limit_strategy: user_data.traffic_limit_strategy.clone(),
                            expire_at: user_data.expire_at.clone(),
                            created_at: Some(user_data.created_at.clone()),
                            last_traffic_reset_at: user_data.last_traffic_reset_at.clone(),
                            description: user_data.description.clone(),
                            tag: user_data.tag.clone(),
                            telegram_id: Some(user_id.clone().parse().unwrap()),
                            email: user_data.email.clone(),
                            hwid_device_limit: user_data.hwid_device_limit.clone(),
                            active_internal_squads: Some(squads),
                        };

                        match client.users.create_user(new_user).await {
                            Ok(user_data) => {
                                log::info!("User {} created successfully (during recreation)", user_id);
                                let keyboard =
                                    InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                                        Messages::ru().back(),
                                        "back_to_main_menu",
                                    )]]);

                                bot.send_message(
                                    chat_id,
                                    format!(
                                        "Новая ссылка на вашу подписку: `{}`",
                                        user_data.response.subscription_url
                                    ),
                                )
                                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                .reply_markup(keyboard)
                                .await?;
                            }
                            Err(e) => {
                                log::error!("Failed to recreate user: {}", e);
                                bot.send_message(
                                    chat_id,
                                    Messages::ru().error("создании пользователя"),
                                )
                                .await?;
                            }
                        };
                    }
                    Err(e) => {
                        log::error!("Failed to delete client: {}", e);
                        let keyboard =
                            InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                                Messages::ru().back(),
                                "back_to_main_menu",
                            )]]);
                        bot.send_message(chat_id, Messages::ru().error("удалении пользователя"))
                            .reply_markup(keyboard)
                            .await?;
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get client info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .await?;
            }
        };
    }

    Ok(())
}

pub async fn back_to_main_menu(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(_msg) = q.message.clone() {
        let user_id = q.from.id.to_string();

        let client = RemnawaveApiClient::new(
            dotenv::var("PANEL_BASE_URL").expect("PANEL_BASE_URL must be set"),
            Some(dotenv::var("REMNAWAVE_API_TOKEN").expect("REMNAWAVE_API_TOKEN must be set")),
        )
        .expect("Failed to create RemnawaveApiClient");

        match client.users.get_user_by_telegram_id(user_id).await {
            Ok(_user) => {
                send_main_menu(&bot, q.chat_id().unwrap()).await?;
            }
            Err(_) => {
                let keyboard =
                    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                        Messages::ru().new_user_confirmed(),
                        "create_new_user",
                    )]]);
                bot.send_message(q.chat_id().unwrap(), Messages::ru().welcome_prompt())
                    .reply_markup(keyboard)
                    .await?;
            }
        };
    }

    Ok(())
}
