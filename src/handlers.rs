use crate::client::get_client;
use crate::error::MyError;
use crate::keyboards;
use crate::messages::Messages;
use crate::types::{Command, HandlerResult};

use remnawave::CreateUserRequestDto;

use teloxide::dispatching::dialogue::GetChatId;
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, Message},
};

use chrono::{TimeZone, Utc};

/// Extracts the user id from a `Message` or returns a default UserId if none exists.
///
/// # Arguments
///
/// * `msg` - The `Message` to extract the user id from.
///
/// # Returns
///
/// The user id as a `UserId`.
fn get_user_id(msg: &Message) -> UserId {
    msg.from.as_ref().map(|user| user.id).unwrap_or(UserId(0))
}

async fn send_main_menu(bot: &Bot, chat_id: ChatId) -> ResponseResult<()> {
    bot.send_message(chat_id, "Главное меню:")
        .reply_markup(keyboards::main_menu())
        .await?;
    Ok(())
}

/// Handles the `/start` command.
///
/// Sends a welcome message and shows the main menu if the user exists, or prompts for creation if not.
///
/// # Arguments
///
/// * `bot` - The bot handle.
/// * `msg` - The received `Message`.
///
/// # Returns
///
/// A `HandlerResult`.
pub async fn start(bot: Bot, msg: Message) -> HandlerResult {
    let user_id = get_user_id(&msg);
    log::info!("User {} called /start", user_id);

    let client = get_client();
    match client
        .users
        .get_user_by_telegram_id(user_id.0.to_string())
        .await
    {
        Ok(_user) => {
            send_main_menu(&bot, msg.chat.id).await?;
        }
        Err(_) => {
            bot.send_message(msg.chat.id, Messages::ru().welcome_prompt())
                .reply_markup(keyboards::new_user_confirmation())
                .await?;
        }
    };
    Ok(())
}

/// Handles the `/help` command by sending a list of available commands to the user.
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
    let user_id = get_user_id(&msg);
    log::info!("User {} called /help", user_id);

    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

/// Handles invalid input by sending an error message to the user.
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
    let user_id = get_user_id(&msg);
    let chat_id = msg.chat.id;
    let user_input = msg.text().unwrap_or_default();

    log::warn!(
        "User {} entered an incorrect value: {}",
        user_id,
        user_input
    );
    bot.send_message(chat_id, Messages::ru().invalid_input())
        .await?;
    Ok(())
}

pub async fn create_new_user(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id;
        let chat_id = msg.chat().id;

        log::info!("User {} called create_new_user", user_id);

        let client = get_client();
        let telegram_id: i64 = user_id
            .0
            .try_into()
            .map_err(|_| MyError::Custom("User ID too large for i64".to_string()))?;
        let new_user = CreateUserRequestDto {
            username: q.from.username.clone().unwrap_or(user_id.to_string()),
            status: remnawave::api::types::common::UserStatus::Active,
            short_uuid: None,
            trojan_password: None,
            vless_uuid: None,
            ss_password: None,
            traffic_limit_bytes: None,
            traffic_limit_strategy: remnawave::api::types::common::TrafficLimitStrategy::NoReset,
            expire_at: Utc.with_ymd_and_hms(2099, 1, 1, 0, 0, 0).unwrap(),
            created_at: None,
            last_traffic_reset_at: None,
            description: None,
            tag: None,
            telegram_id: Some(telegram_id),
            email: None,
            hwid_device_limit: None,
            active_internal_squads: None,
        };

        match client.users.create_user(new_user).await {
            Ok(user_data) => {
                log::info!("User {} created successfully", user_id);
                bot.send_message(
                    chat_id,
                    format!(
                        "Ваша подписка создана\\! Ссылка: `{}`",
                        user_data.response.subscription_url
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
            Err(e) => {
                log::error!("Failed to create user: {}", e);
                bot.send_message(chat_id, Messages::ru().error("создании пользователя"))
                    .reply_markup(keyboards::back_to_main_menu())
                    .await?;
            }
        };
    }
    Ok(())
}

pub async fn recreate_sub_link(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id;
        let chat_id = msg.chat().id;

        log::info!("User {} called recreate_sub_link", user_id);

        let client = get_client();
        match client
            .users
            .get_user_by_telegram_id(user_id.0.to_string())
            .await
        {
            Ok(user) => {
                let user_data = &user.response[0];
                let user_uuid = user_data.uuid;

                match client.users.delete_user(user_uuid).await {
                    Ok(_) => {
                        log::info!("User {} deleted successfully (during recreation)", user_id);
                        let squads: Vec<String> = user_data
                            .active_internal_squads
                            .iter()
                            .map(|s| s.clone().uuid.to_string())
                            .collect();
                        let telegram_id: i64 = user_id.0.try_into().map_err(|_| {
                            MyError::Custom("User ID too large for i64".to_string())
                        })?;
                        let new_user = CreateUserRequestDto {
                            username: user_data.username.clone(),
                            status: user_data.status.clone(),
                            short_uuid: None,
                            trojan_password: None,
                            vless_uuid: None,
                            ss_password: None,
                            traffic_limit_bytes: Some(0),
                            traffic_limit_strategy: user_data.traffic_limit_strategy.clone(),
                            expire_at: user_data.expire_at,
                            created_at: Some(user_data.created_at),
                            last_traffic_reset_at: user_data.last_traffic_reset_at,
                            description: user_data.description.clone(),
                            tag: user_data.tag.clone(),
                            telegram_id: Some(telegram_id),
                            email: user_data.email.clone(),
                            hwid_device_limit: user_data.hwid_device_limit,
                            active_internal_squads: Some(squads),
                        };

                        match client.users.create_user(new_user).await {
                            Ok(user_data) => {
                                log::info!(
                                    "User {} created successfully (during recreation)",
                                    user_id
                                );
                                bot.send_message(
                                    chat_id,
                                    format!(
                                        "Новая ссылка на вашу подписку: `{}`",
                                        user_data.response.subscription_url
                                    ),
                                )
                                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                .reply_markup(keyboards::back_to_main_menu())
                                .await?;
                            }
                            Err(e) => {
                                log::error!("Failed to recreate user: {}", e);
                                bot.send_message(
                                    chat_id,
                                    Messages::ru().error("создании пользователя"),
                                )
                                .reply_markup(keyboards::back_to_main_menu())
                                .await?;
                            }
                        };
                    }
                    Err(e) => {
                        log::error!("Failed to delete client: {}", e);
                        bot.send_message(chat_id, Messages::ru().error("удалении пользователя"))
                            .reply_markup(keyboards::back_to_main_menu())
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
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
        };
    }
    Ok(())
}

pub async fn back_to_main_menu(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(ref _msg) = q.message {
        let user_id = q.from.id;
        log::info!("User {} called back_to_main_menu", user_id);

        let client = get_client();
        match client
            .users
            .get_user_by_telegram_id(user_id.0.to_string())
            .await
        {
            Ok(_user) => {
                send_main_menu(&bot, q.chat_id().unwrap()).await?;
            }
            Err(_) => {
                bot.send_message(q.chat_id().unwrap(), Messages::ru().welcome_prompt())
                    .reply_markup(keyboards::new_user_confirmation())
                    .await?;
            }
        };
    }
    Ok(())
}

pub async fn show_about_me(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id;
        let chat_id = msg.chat().id;
        log::info!("User {} called show_about_me", user_id);

        let client = get_client();
        match client
            .users
            .get_user_by_telegram_id(user_id.0.to_string())
            .await
        {
            Ok(user) => {
                let user_data = &user.response[0];

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
                    .reply_markup(keyboards::back_to_main_menu())
                    .await?;
            }
            Err(e) => {
                log::error!("Failed to get user info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
        };
    }
    Ok(())
}

pub async fn delete_me(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id;
        let chat_id = msg.chat().id;
        log::info!("User {} called delete_me", user_id);

        let client = get_client();
        match client
            .users
            .get_user_by_telegram_id(user_id.0.to_string())
            .await
        {
            Ok(user) => {
                let user_uuid = user.response[0].uuid;
                match client.users.delete_user(user_uuid).await {
                    Ok(_) => {
                        log::info!("User {} deleted successfully", user_id);
                        bot.send_message(chat_id, "Ваша подписка успешно удалена, для повторного создания подписки используйте команду /start")
                            .await?;
                    }
                    Err(e) => {
                        log::error!("Failed to delete user: {}", e);
                        bot.send_message(chat_id, Messages::ru().error("удалении пользователя"))
                            .reply_markup(keyboards::back_to_main_menu())
                            .await?;
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get user info: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о пользователе"),
                )
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
        };
    }
    Ok(())
}

pub async fn show_sub_link(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let user_id = q.from.id;
        log::info!("User {} called show_sub_link", user_id);

        let client = get_client();
        match client
            .users
            .get_user_by_telegram_id(user_id.0.to_string())
            .await
        {
            Ok(user) => {
                bot.send_message(
                    msg.chat().id,
                    format!(
                        "Ваша ссылка на подписку: `{}`",
                        user.response[0].subscription_url
                    ),
                )
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
            Err(e) => {
                log::error!("Failed to get subscription link: {}", e);
                bot.send_message(
                    msg.chat().id,
                    Messages::ru().error("получении ссылки на подписку"),
                )
                .reply_markup(keyboards::back_to_main_menu())
                .await?;
            }
        };
    }
    Ok(())
}
