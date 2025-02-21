use super::error::MyError;
use super::types::{Command, HandlerResult, MyDialogue, State};
use super::xui_api::ThreeXUiClient;
use fast_qr::convert::{image::ImageBuilder, Builder, Shape};
use fast_qr::qr::QRBuilder;
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};
use uuid::Uuid;

const MAX_DEVICES: u8 = 5;
const SUPPORTED_PLATFORMS: [&str; 5] = ["Windows", "Android", "Linux", "MacOS", "iOS"];

fn get_username(msg: &Message) -> &str {
    msg.from
        .as_ref()
        .and_then(|user| user.username.as_deref())
        .unwrap_or("unknown")
}

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = get_username(&msg);
    let chat_id = msg.chat.id;

    log::info!("User {} (chat_id={}) called /start", username, chat_id);

    const WELCOME_MESSAGE: &str = "👋 Привет! Я помогу вам подключиться к GlebusVPN. 🚀\n\n\
                                   Введите количество подключаемых устройств (1-5):";

    bot.send_message(chat_id, WELCOME_MESSAGE).await?;
    dialogue.update(State::ReceiveDeviceCount).await?;

    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let username = get_username(&msg);
    let chat_id = msg.chat.id;

    log::info!("User {} (chat_id={}) called /help", username, chat_id);

    bot.send_message(chat_id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = get_username(&msg);
    let chat_id = msg.chat.id;

    log::info!("User {} (chat_id={}) called /cancel", username, chat_id);

    bot.send_message(chat_id, "❌ Отменяем текущую операцию.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    let username = get_username(&msg);
    let chat_id = msg.chat.id;
    let user_input = msg.text().unwrap_or_default();

    log::warn!(
        "User {} (chat_id={}) entered an incorrect value: {}",
        username,
        chat_id,
        user_input
    );

    bot.send_message(
        chat_id,
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\nИспользуйте /help для справки. 😊",
    )
    .await?;
    Ok(())
}

pub async fn receive_device_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let user_input = msg.text().unwrap_or_default();
    let username = get_username(&msg);
    let chat_id = msg.chat.id;

    match user_input.parse::<u8>() {
        Ok(count) if (1..=MAX_DEVICES).contains(&count) => {
            handle_valid_device_count(bot, dialogue, username, chat_id, count).await
        }
        Ok(count) if count > MAX_DEVICES => {
            handle_excessive_device_count(bot, username, chat_id, count).await
        }
        _ => handle_invalid_device_count(bot, username, chat_id, user_input).await,
    }
}

async fn handle_valid_device_count(
    bot: Bot,
    dialogue: MyDialogue,
    username: &str,
    chat_id: ChatId,
    count: u8,
) -> HandlerResult {
    log::info!(
        "User {} (chat_id={}) started VPN setup for {} devices",
        username,
        dialogue.chat_id(),
        count
    );

    bot.send_message(
        chat_id,
        "🚀 Отлично! Теперь укажите, пожалуйста, платформу каждого устройства.",
    )
    .await?;
    dialogue
        .update(State::ReceiveDeviceInfo {
            total_devices: count,
            current_device: 1,
            applications: Vec::new(),
        })
        .await?;
    ask_device_platform(&bot, chat_id, 1).await
}

async fn handle_excessive_device_count(
    bot: Bot,
    username: &str,
    chat_id: ChatId,
    count: u8,
) -> HandlerResult {
    log::warn!(
        "User {} (chat_id={}) entered an excessive amount of devices: {}",
        username,
        chat_id,
        count
    );

    bot.send_message(chat_id, "❌ Максимальное количество устройств — 5. 😔\n\nЕсли вам нужно больше, обратитесь к администратору @LineGM. Спасибо за понимание! 🙌").await?;
    Ok(())
}

async fn handle_invalid_device_count(
    bot: Bot,
    username: &str,
    chat_id: ChatId,
    user_input: &str,
) -> HandlerResult {
    log::warn!(
        "User {} (chat_id={}) entered an incorrect amount of devices: {}",
        username,
        chat_id,
        user_input
    );

    bot.send_message(chat_id, "⚠️ Пожалуйста, введите число от 1 до 5. 🚀")
        .await?;
    Ok(())
}

async fn ask_device_platform(bot: &Bot, chat_id: ChatId, device_num: u8) -> HandlerResult {
    let platforms = SUPPORTED_PLATFORMS
        .iter()
        .map(|&p| InlineKeyboardButton::callback(p, p))
        .collect::<Vec<_>>();

    bot.send_message(chat_id, format!("📱 Устройство #{}:", device_num))
        .reply_markup(InlineKeyboardMarkup::new([platforms]))
        .await?;
    Ok(())
}

pub async fn receive_platform_selection(
    bot: Bot,
    dialogue: MyDialogue,
    (total_devices, current_device, mut applications): (u8, u8, Vec<String>),
    q: CallbackQuery,
) -> HandlerResult {
    let Some(platform) = &q.data else {
        return Ok(());
    };

    let username = q.from.username.as_deref().unwrap_or("unknown");
    let chat_id = dialogue.chat_id();

    log::info!(
        "User {} (chat_id={}) selected {} for device {}",
        username,
        chat_id,
        platform,
        current_device
    );
    applications.push(format!("Device {}: {}", current_device, platform));

    if !handle_api_operations(&bot, &dialogue, username, platform).await? {
        return Ok(());
    }

    if current_device < total_devices {
        handle_next_device(&bot, dialogue, total_devices, current_device, applications).await?;
    } else {
        handle_completion(&bot, dialogue, username).await?;
    }

    Ok(())
}

async fn handle_api_operations(
    bot: &Bot,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> Result<bool, MyError> {
    let base_url = dotenv::var("PANEL_BASE_URL")?;
    let mut api = ThreeXUiClient::new(&base_url);

    let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
    let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

    if !try_login(&mut api, &admin_login, &admin_password).await {
        send_error_message(bot, dialogue, "панели сервера").await?;
        dialogue.exit().await?;
        return Ok(false);
    }

    if !try_add_client(&mut api, dialogue, username, platform).await {
        send_error_message(bot, dialogue, "добавлении очередного подключения").await?;
        dialogue.exit().await?;
        return Ok(false);
    }

    send_connection_info(bot, dialogue, username, platform).await?;
    Ok(true)
}

async fn try_login(api: &mut ThreeXUiClient, login: &str, password: &str) -> bool {
    match api.login(login, password).await {
        Ok(()) => {
            log::info!("Login as {} successfully.", login);
            true
        }
        Err(err) => {
            log::error!("Login as {} failed with status: {}", login, err);
            false
        }
    }
}

async fn try_add_client(
    api: &mut ThreeXUiClient,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> bool {
    let client_id = format!("{}_{}", username, platform.to_lowercase());
    let new_client = serde_json::json!({
        "clients": [{
            "id": Uuid::new_v4().simple().to_string(),
            "email": client_id,
            "comment": "Added through GlebusVPN bot.",
            "flow": "xtls-rprx-vision",
            "enable": true,
            "tgId": dialogue.chat_id(),
            "subId": client_id
        }]
    });

    match api.add_client(1, &new_client).await {
        Ok(json) => {
            log::info!("Add client result: {}", json);
            true
        }
        Err(json) => {
            log::error!("Add client result: {}", json);
            false
        }
    }
}

async fn send_error_message(
    bot: &Bot,
    dialogue: &MyDialogue,
    error_context: &str,
) -> HandlerResult {
    bot.send_message(dialogue.chat_id(), format!("⚠️ Ой, кажется, в {} что-то пошло не так. 😕\n\nПопробуйте ещё раз. 🔄\n\nЕсли это не поможет, то свяжитесь с администратором.", error_context)).await?;
    Ok(())
}

async fn send_connection_info(
    bot: &Bot,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> HandlerResult {
    let client_id = format!("{}_{}", username, platform.to_lowercase());
    let sub_url = format!("{}/{}", dotenv::var("SUB_BASE_URL")?, client_id);
    let temp_dir = std::env::temp_dir();
    let image_name = temp_dir.join(format!("{}.png", client_id));

    let qrcode = QRBuilder::new(sub_url.clone()).build()?;
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color([255, 255, 255, 0])
        .fit_width(600)
        .to_file(&qrcode, image_name.to_str().ok_or("Invalid path encoding")?)?;

    bot.send_photo(
        dialogue.chat_id(),
        teloxide::types::InputFile::file(&image_name),
    )
    .await?;

    if let Err(e) = std::fs::remove_file(&image_name) {
        log::warn!("Failed to remove temporary QR code file: {}", e);
    }

    bot.send_message(dialogue.chat_id(), format!("`{}`\n\nВставьте эту ссылку в приложение Hiddify, оно есть на всех предложенных платформах", &sub_url))
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

async fn handle_next_device(
    bot: &Bot,
    dialogue: MyDialogue,
    total_devices: u8,
    current_device: u8,
    applications: Vec<String>,
) -> HandlerResult {
    let next_device = current_device + 1;
    dialogue
        .update(State::ReceiveDeviceInfo {
            total_devices,
            current_device: next_device,
            applications,
        })
        .await?;
    ask_device_platform(bot, dialogue.chat_id(), next_device).await
}

async fn handle_completion(bot: &Bot, dialogue: MyDialogue, username: &str) -> HandlerResult {
    log::info!(
        "User {} (chat_id={}) successfully completed the request",
        username,
        dialogue.chat_id()
    );

    bot.send_message(
        dialogue.chat_id(),
        "🎉 Поздравляем! Ваши подключения успешно созданы. ✅",
    )
    .await?;
    dialogue.exit().await?;
    Ok(())
}
