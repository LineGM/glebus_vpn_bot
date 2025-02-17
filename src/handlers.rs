use super::types::{Command, HandlerResult, MyDialogue, State};
use super::xui_api::ThreeXUiClient;
use fast_qr::convert::{image::ImageBuilder, Builder, Shape};
use fast_qr::qr::QRBuilder;
#[allow(unused_imports)]
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};
use uuid::Uuid;

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");
    log::info!("User {} (chat_id={}) called /start", username, msg.chat.id);

    bot.send_message(
        msg.chat.id,
        "👋 Привет! Я помогу вам подключиться к GlebusVPN. 🚀\n\nВведите количество подключаемых устройств (1-5):"
    )
    .await
    .map(|_| ())?;

    dialogue.update(State::ReceiveDeviceCount).await?;
    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");
    log::info!("User {} (chat_id={}) called /help", username, msg.chat.id);

    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await
        .map(|_| ())?;
    Ok(())
}

pub async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");
    log::info!("User {} (chat_id={}) called /cancel", username, msg.chat.id);

    bot.send_message(msg.chat.id, "❌ Отменяем текущую операцию.")
        .await
        .map(|_| ())?;
    dialogue.exit().await?;
    Ok(())
}

pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    let user_input = msg.text().unwrap_or_default();
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");

    log::warn!(
        "User {} (chat_id={}) entered an incorrect value: {}",
        username,
        msg.chat.id,
        user_input
    );

    bot.send_message(
        msg.chat.id,
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\nИспользуйте /help для справки. 😊",
    )
    .await
    .map(|_| ())?;
    Ok(())
}

pub async fn receive_device_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let user_input = msg.text().unwrap_or_default();
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");

    match user_input.parse::<u8>() {
        Ok(count) if (1..=5).contains(&count) => {
            log::info!(
                "User {} (chat_id={}) started VPN setup for {} devices",
                username,
                dialogue.chat_id(),
                count
            );

            bot.send_message(
                msg.chat.id,
                "🚀 Отлично! Теперь укажите, пожалуйста, платформу каждого устройства.",
            )
            .await
            .map(|_| ())?;

            dialogue
                .update(State::ReceiveDeviceInfo {
                    total_devices: count,
                    current_device: 1,
                    applications: Vec::new(),
                })
                .await?;

            ask_device_platform(&bot, msg.chat.id, 1).await
        }
        Ok(count) if count > 5 => {
            log::warn!(
                "User {} (chat_id={}) entered an excessive amount of devices: {}",
                username,
                msg.chat.id,
                count
            );

            bot.send_message(
                msg.chat.id,
                "❌ Максимальное количество устройств — 5. 😔\n\nЕсли вам нужно больше, обратитесь к администратору @LineGM. Спасибо за понимание! 🙌"
            )
            .await
            .map(|_| ())?;
            Ok(())
        }
        _ => {
            log::warn!(
                "User {} (chat_id={}) entered an incorrect amount of devices: {}",
                username,
                msg.chat.id,
                user_input
            );

            bot.send_message(msg.chat.id, "⚠️ Пожалуйста, введите число от 1 до 5. 🚀")
                .await
                .map(|_| ())?;
            Ok(())
        }
    }
}

async fn ask_device_platform(bot: &Bot, chat_id: ChatId, device_num: u8) -> HandlerResult {
    let platforms = ["Windows", "Android", "Linux", "MacOS", "iOS"]
        .map(|p| InlineKeyboardButton::callback(p, p));

    bot.send_message(chat_id, format!("📱 Устройство #{}:", device_num))
        .reply_markup(InlineKeyboardMarkup::new([platforms]))
        .await
        .map(|_| ())?;
    Ok(())
}

pub async fn receive_platform_selection(
    bot: Bot,
    dialogue: MyDialogue,
    (total_devices, current_device, mut applications): (u8, u8, Vec<String>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(platform) = &q.data {
        let username = q.from.username.as_deref().unwrap_or("unknown");
        log::info!(
            "User {} (chat_id={}) selected {} for device {}",
            username,
            dialogue.chat_id(),
            platform,
            current_device
        );

        applications.push(format!("Device {}: {}", current_device, platform));

        let base_url = dotenv::var("PANEL_BASE_URL").unwrap();
        let mut _api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN").unwrap();
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD").unwrap();

        let login_result = match _api.login(&admin_login, &admin_password).await {
            Ok(()) => {
                log::info!("Login as {} succesfully.", admin_login);
                true
            }
            Err(err) => {
                log::info!("Login as {} failed with status: {}", admin_login, err);
                false
            }
        };

        if !login_result {
            bot.send_message(
                dialogue.chat_id(),
                "⚠️ Ой, кажется, при доступе к панели сервера что-то пошло не так. 😕\n\nПопробуйте ещё раз. 🔄\n\nЕсли это не поможет, то свяжитесь с администратором.",
            )
            .await
            .map(|_| ())?;

            dialogue.exit().await?;

            return Ok(());
        }

        let new_client = serde_json::json!({
            "clients": [{
                "id": Uuid::new_v4().simple().to_string(),
                "email": username.to_owned() + "_" + &platform.to_lowercase(),
                "comment": "Added through GlebusVPN bot.",
                "flow": "xtls-rprx-vision",
                "enable": true,
                "tgId": dialogue.chat_id(),
                "subId": username.to_owned() + "_" + &platform.to_lowercase()
            }]
        });

        let add_client_result = match _api.add_client(1, &new_client).await {
            Ok(json) => {
                log::info!("Add client result: {}", json);
                true
            }
            Err(json) => {
                log::error!("Add client result: {}", json);
                false
            }
        };

        if !add_client_result {
            bot.send_message(
                dialogue.chat_id(),
                "⚠️ Ой, кажется, при добавлении очередного подключения что-то пошло не так. 😕\n\nПопробуйте ещё раз. 🔄\n\nЕсли это не поможет, то свяжитесь с администратором.",
            )
            .await
            .map(|_| ())?;

            dialogue.exit().await?;

            return Ok(());
        }

        let sub_url = format!(
            "{}/{}",
            dotenv::var("SUB_BASE_URL").unwrap(),
            username.to_owned() + "_" + &platform.to_lowercase()
        );

        let qrcode = QRBuilder::new(sub_url.clone()).build().unwrap();
        let image_name = username.to_owned() + "_" + &platform.to_lowercase() + ".png";

        let _img = ImageBuilder::default()
            .shape(Shape::RoundedSquare)
            .background_color([255, 255, 255, 0])
            .fit_width(600)
            .to_file(&qrcode, &image_name);

        bot.send_photo(
            dialogue.chat_id(),
            teloxide::types::InputFile::file(image_name),
        )
        .await
        .map(|_| ())?;

        bot.send_message(
            dialogue.chat_id(),
            format!("`{}`\n\nВставьте эту ссылку в приложение Hiddify, оно есть на всех предложенных платформах", &sub_url)
        )
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await
        .map(|_| ())?;

        if current_device < total_devices {
            let next_device = current_device + 1;
            dialogue
                .update(State::ReceiveDeviceInfo {
                    total_devices,
                    current_device: next_device,
                    applications,
                })
                .await?;

            ask_device_platform(&bot, dialogue.chat_id(), next_device).await?;
        } else {
            log::info!(
                "User {} (chat_id={}) successfully completed the request",
                match q.from.username.as_ref() {
                    Some(username) => username,
                    None => "с неизвестным username",
                },
                dialogue.chat_id(),
            );

            bot.send_message(
                dialogue.chat_id(),
                "🎉 Поздравляем! Ваши подключения успешно созданы. ✅",
            )
            .await
            .map(|_| ())?;

            dialogue.exit().await?;
        }
    }
    Ok(())
}
