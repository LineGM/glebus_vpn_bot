use crate::client::get_client;
use crate::error::MyError;
use crate::keyboards;
use crate::messages::Messages;
use crate::types::{Command, HandlerResult};
use chrono::{TimeZone, Utc};
use remnawave::CreateUserRequestDto;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, Message, MessageId},
};

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

async fn send_main_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: Option<MessageId>,
) -> ResponseResult<()> {
    if let Some(mid) = message_id {
        bot.edit_message_text(chat_id, mid, "Главное меню:")
            .reply_markup(keyboards::main_menu())
            .await?;
    } else {
        bot.send_message(chat_id, "Главное меню:")
            .reply_markup(keyboards::main_menu())
            .await?;
    }
    Ok(())
}

/// Sends or edits an error message with back button.
///
/// If message_id is Some, edits the existing message; otherwise sends a new one.
async fn send_error(
    bot: &Bot,
    chat_id: ChatId,
    context: &str,
    message_id: Option<MessageId>,
) -> ResponseResult<()> {
    let error_msg = Messages::ru().error(context);
    if let Some(mid) = message_id {
        bot.edit_message_text(chat_id, mid, error_msg)
            .reply_markup(keyboards::back_to_main_menu())
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    } else {
        bot.send_message(chat_id, error_msg)
            .reply_markup(keyboards::back_to_main_menu())
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await?;
    }
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
            send_main_menu(&bot, msg.chat.id, None).await?;
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

/// Unified handler for all callback queries.
///
/// Dispatches the callback based on the data in the query.
pub async fn handle_callback(bot: Bot, q: CallbackQuery) -> HandlerResult {
    let data = q.data.as_deref().unwrap_or("");
    let result = match data {
        "create_new_user" => create_new_user(&bot, &q).await,
        "show_about_me" => show_about_me(&bot, &q).await,
        "show_sub_link" => show_sub_link(&bot, &q).await,
        "recreate_sub_link" => recreate_sub_link(&bot, &q).await,
        "delete_me" => delete_me(&bot, &q).await,
        "back_to_main_menu" => back_to_main_menu(&bot, &q).await,
        _ => {
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), "Неизвестная команда.")
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                bot.send_message(chat_id, "Неизвестная команда.").await?;
            }
            Ok(())
        }
    };

    if let Err(e) = result {
        log::error!("Callback error: {}", e);
        if let Some(ref msg) = q.message {
            send_error(
                &bot,
                q.chat_id().unwrap(),
                "обработке запроса",
                Some(msg.id()),
            )
            .await?;
        } else if let Some(chat_id) = q.chat_id() {
            send_error(&bot, chat_id, "обработке запроса", None).await?;
        }
    }
    Ok(())
}

async fn create_new_user(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
    log::info!("User {} called create_new_user", user_id);

    let telegram_id: i64 = user_id
        .0
        .try_into()
        .map_err(|_| MyError::Custom("User ID too large for i64".to_string()))?;

    let client = get_client();
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
            let success_msg = format!(
                "Ваша подписка создана\\! Ссылка: `{}`",
                user_data.response.subscription_url
            );
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), success_msg)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                bot.send_message(chat_id, success_msg)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
        }
        Err(e) => {
            log::error!("Failed to create user: {}", e);
            if let Some(ref msg) = q.message {
                send_error(
                    bot,
                    q.chat_id().unwrap(),
                    "создании пользователя",
                    Some(msg.id()),
                )
                .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_error(bot, chat_id, "создании пользователя", None).await?;
            }
        }
    };
    Ok(())
}

async fn recreate_sub_link(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
    log::info!("User {} called recreate_sub_link", user_id);

    let telegram_id: i64 = user_id
        .0
        .try_into()
        .map_err(|_| MyError::Custom("User ID too large for i64".to_string()))?;

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
                            log::info!("User {} created successfully (during recreation)", user_id);
                            let success_msg = format!(
                                "Новая ссылка на вашу подписку: `{}`",
                                user_data.response.subscription_url
                            );
                            if let Some(ref msg) = q.message {
                                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), success_msg)
                                    .reply_markup(keyboards::back_to_main_menu())
                                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                    .await?;
                            } else if let Some(chat_id) = q.chat_id() {
                                bot.send_message(chat_id, success_msg)
                                    .reply_markup(keyboards::back_to_main_menu())
                                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                    .await?;
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to recreate user: {}", e);
                            if let Some(ref msg) = q.message {
                                send_error(
                                    bot,
                                    q.chat_id().unwrap(),
                                    "создании пользователя",
                                    Some(msg.id()),
                                )
                                .await?;
                            } else if let Some(chat_id) = q.chat_id() {
                                send_error(bot, chat_id, "создании пользователя", None).await?;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to delete client: {}", e);
                    if let Some(ref msg) = q.message {
                        send_error(
                            bot,
                            q.chat_id().unwrap(),
                            "удалении пользователя",
                            Some(msg.id()),
                        )
                        .await?;
                    } else if let Some(chat_id) = q.chat_id() {
                        send_error(bot, chat_id, "удалении пользователя", None).await?;
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get client info: {}", e);
            if let Some(ref msg) = q.message {
                send_error(
                    bot,
                    q.chat_id().unwrap(),
                    "получении информации о пользователе",
                    Some(msg.id()),
                )
                .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_error(bot, chat_id, "получении информации о пользователе", None).await?;
            }
        }
    };
    Ok(())
}

async fn back_to_main_menu(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
    log::info!("User {} called back_to_main_menu", user_id);

    let client = get_client();
    match client
        .users
        .get_user_by_telegram_id(user_id.0.to_string())
        .await
    {
        Ok(_user) => {
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), "Главное меню:")
                    .reply_markup(keyboards::main_menu())
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_main_menu(bot, chat_id, None).await?;
            }
        }
        Err(_) => {
            let welcome_msg = Messages::ru().welcome_prompt();
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), welcome_msg)
                    .reply_markup(keyboards::new_user_confirmation())
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                bot.send_message(chat_id, welcome_msg)
                    .reply_markup(keyboards::new_user_confirmation())
                    .await?;
            }
        }
    };
    Ok(())
}

async fn show_about_me(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
    log::info!("User {} called show_about_me", user_id);

    let client = get_client();
    match client
        .users
        .get_user_by_telegram_id(user_id.0.to_string())
        .await
    {
        Ok(user) => {
            let user_data = &user.response[0];
            let info = format!(
                "🔑 *Профиль пользователя*\n Имя пользователя: `{}`\n Статус: `{}`\n📲 *Идентификаторы*\n Telegram ID: `{}`\n Email: `{}`\n📊 *Трафик*\n Использовано за все время: `{}`\n Лимит трафика: `{}`\n🖥 *Подключения и агенты*\n Последний UserAgent: `{}`\n Первое подключение: `{}`\n⏰ *Срок действия подписки*\n Активно до: `{}`\n📥 *Ссылки*\n Подписка: `{}`\n HAPP Crypto Link: `{}`",
                user_data.username,
                user_data.status,
                user_data.telegram_id.unwrap_or(0),
                user_data.email.clone().unwrap_or("null".to_string()),
                user_data.lifetime_used_traffic_bytes,
                user_data.traffic_limit_bytes,
                user_data
                    .sub_last_user_agent
                    .clone()
                    .unwrap_or("null".to_string()),
                user_data.first_connected_at.unwrap_or_default(),
                user_data.expire_at,
                user_data.subscription_url,
                user_data.happ.crypto_link
            );
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), info)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                bot.send_message(chat_id, info)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
        }
        Err(e) => {
            log::error!("Failed to get user info: {}", e);
            if let Some(ref msg) = q.message {
                send_error(
                    bot,
                    q.chat_id().unwrap(),
                    "получении информации о пользователе",
                    Some(msg.id()),
                )
                .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_error(bot, chat_id, "получении информации о пользователе", None).await?;
            }
        }
    };
    Ok(())
}

async fn delete_me(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
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
                    let success_msg = "Ваша подписка успешно удалена, для повторного создания подписки используйте команду /start";
                    if let Some(ref msg) = q.message {
                        bot.edit_message_text(q.chat_id().unwrap(), msg.id(), success_msg)
                            .await?;
                    } else if let Some(chat_id) = q.chat_id() {
                        bot.send_message(chat_id, success_msg).await?;
                    }
                }
                Err(e) => {
                    log::error!("Failed to delete user: {}", e);
                    if let Some(ref msg) = q.message {
                        send_error(
                            bot,
                            q.chat_id().unwrap(),
                            "удалении пользователя",
                            Some(msg.id()),
                        )
                        .await?;
                    } else if let Some(chat_id) = q.chat_id() {
                        send_error(bot, chat_id, "удалении пользователя", None).await?;
                    }
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get user info: {}", e);
            if let Some(ref msg) = q.message {
                send_error(
                    bot,
                    q.chat_id().unwrap(),
                    "получении информации о пользователе",
                    Some(msg.id()),
                )
                .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_error(bot, chat_id, "получении информации о пользователе", None).await?;
            }
        }
    };
    Ok(())
}

async fn show_sub_link(bot: &Bot, q: &CallbackQuery) -> HandlerResult {
    let user_id = q.from.id;
    log::info!("User {} called show_sub_link", user_id);

    let client = get_client();
    match client
        .users
        .get_user_by_telegram_id(user_id.0.to_string())
        .await
    {
        Ok(user) => {
            let success_msg = format!(
                "Ваша ссылка на подписку: `{}`",
                user.response[0].subscription_url
            );
            if let Some(ref msg) = q.message {
                bot.edit_message_text(q.chat_id().unwrap(), msg.id(), success_msg)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            } else if let Some(chat_id) = q.chat_id() {
                bot.send_message(chat_id, success_msg)
                    .reply_markup(keyboards::back_to_main_menu())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
        }
        Err(e) => {
            log::error!("Failed to get subscription link: {}", e);
            if let Some(ref msg) = q.message {
                send_error(
                    bot,
                    q.chat_id().unwrap(),
                    "получении ссылки на подписку",
                    Some(msg.id()),
                )
                .await?;
            } else if let Some(chat_id) = q.chat_id() {
                send_error(bot, chat_id, "получении ссылки на подписку", None).await?;
            }
        }
    };
    Ok(())
}
