use super::types::{Command, HandlerResult, MyDialogue, State};
#[allow(unused_imports)]
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup},
};

pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg
        .from
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("unknown");
    log::info!("User {} (chat_id={}) called /start", username, msg.chat.id);

    bot.send_message(
        msg.chat.id,
        "👋 Привет! Добро пожаловать в наш бот. Мы поможем вам подключиться к GlebusVPN. 🚀\n\nВведите количество устройств (1-5):"
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
                "🚀 Отлично! Давайте начнём вводить данные для каждого устройства.",
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
                "🎉 Поздравляем! Ваша заявка успешно сформирована. ✅",
            )
            .await
            .map(|_| ())?;

            dialogue.exit().await?;
        }
    }
    Ok(())
}
