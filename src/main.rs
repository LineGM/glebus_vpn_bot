use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
#[command(rename_rule = "lowercase")]
enum Command {
    Help,
    Start,
    Cancel,
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveDeviceCount].endpoint(receive_device_count))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceiveDeviceInfo {
            total_devices,
            current_device,
            applications
        }]
        .endpoint(receive_platform_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = msg
        .from
        .map(|u| u.username.unwrap_or_else(|| "неизвестный".to_string()))
        .unwrap_or_else(|| "аноним".to_string());

    log::info!(
        "Пользователь {} с chat_id={} вызвал команду /start",
        username,
        msg.chat.id
    );

    bot.send_message(
        msg.chat.id,
        "👋 Привет! Добро пожаловать в наш бот. Мы поможем вам подключиться к GlebusVPN. 🚀\n\nВведите количество устройств, для которых вы хотите оформить доступ (от 1 до 5):",
    )
    .await?;
    dialogue.update(State::ReceiveDeviceCount).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    log::info!(
        "Пользователь {} с chat_id={} вызвал команду /help",
        msg.from
            .map(|u| u.username.unwrap_or_else(|| "неизвестный".to_string()))
            .unwrap_or_else(|| "аноним".to_string()),
        msg.chat.id
    );

    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    log::info!(
        "Пользователь {} с chat_id={} вызвал команду /cancel",
        msg.from
            .map(|u| u.username.unwrap_or_else(|| "неизвестный".to_string()))
            .unwrap_or_else(|| "аноним".to_string()),
        msg.chat.id
    );

    bot.send_message(msg.chat.id, "❌ Отменяем текущую операцию.")
        .await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\nИспользуйте /help для справки. 😊",
    )
    .await?;
    Ok(())
}

async fn receive_device_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let user_input = msg.text().unwrap_or("неизвестно");
    let username = msg
        .from
        .as_ref()
        .map(|u| {
            u.username
                .clone()
                .unwrap_or_else(|| "неизвестный".to_string())
        })
        .unwrap_or_else(|| "аноним".to_string());

    match user_input.parse::<u8>() {
        Ok(count) if count > 0 && count <= 5 => {
            log::info!(
                "Пользователь {} с chat_id={} начал оформление заявки на {} устройств",
                username,
                msg.chat.id,
                count
            );

            bot.send_message(
                msg.chat.id,
                "🚀 Отлично! Давайте начнём вводить данные для каждого устройства. Пожалуйста, следуйте инструкциям ниже. 😊",
            )
            .await?;

            dialogue
                .update(State::ReceiveDeviceInfo {
                    total_devices: count,
                    current_device: 1,
                    applications: Vec::new(),
                })
                .await?;

            ask_device_platform(&bot, msg.chat.id, 1).await?;
        }
        Ok(count) if count > 5 => {
            log::warn!(
                "Пользователь {} с chat_id={} ввёл превышающее значение устройств: {}",
                username,
                msg.chat.id,
                count
            );

            bot.send_message(
                msg.chat.id,
                format!(
                    "❌ Увы, максимальное количество устройств для оформления — 5. 😔\n\nЕсли вам нужно больше, обратитесь к администратору @LineGM. Спасибо за понимание! 🙌"
                )
            )
            .await?;
        }
        Ok(_) => {
            log::warn!(
                "Пользователь {} с chat_id={} ввёл некорректное значение: {}",
                username,
                msg.chat.id,
                user_input
            );

            bot.send_message(
                msg.chat.id,
                "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\nПопробуйте ещё раз! Пожалуйста, введите число от 1 до 5. 🚀"
            )
            .await?;
        }
        Err(_) => {
            log::warn!(
                "Пользователь {} с chat_id={} ввёл некорректное значение: {}",
                username,
                msg.chat.id,
                user_input
            );

            bot.send_message(
                msg.chat.id,
                "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\nПопробуйте ещё раз! Пожалуйста, введите число от 1 до 5. 🚀",
            )
            .await?;
        }
    }
    Ok(())
}

async fn ask_device_platform(bot: &Bot, chat_id: ChatId, device_num: u8) -> HandlerResult {
    let platforms = ["Windows", "Android", "Linux", "MacOS", "iOS"]
        .map(|p| InlineKeyboardButton::callback(p, p));

    bot.send_message(chat_id, format!("📱 Устройство #{}:", device_num))
        .reply_markup(InlineKeyboardMarkup::new([platforms]))
        .await?;
    Ok(())
}

async fn receive_platform_selection(
    bot: Bot,
    dialogue: MyDialogue,
    (total_devices, current_device, mut applications): (u8, u8, Vec<String>),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(platform) = &q.data {
        log::info!(
            "Пользователь с chat_id={} выбрал платформу {} для устройства {}",
            q.from.id,
            platform,
            current_device
        );

        applications.push(format!("Устройство {}: {}", current_device, platform));

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
            let summary = applications.join("\n");
            log::info!(
                "Пользователь с chat_id={} завершил оформление заявки:\n{}",
                q.from.id,
                summary
            );

            bot.send_message(
                q.from.id,
                format!(
                    "🎉 Поздравляем! Ваша заявка успешно сформирована. ✅\n\nСпасибо за использование нашего сервиса! 🙏",
                ),
            )
            .await?;

            dialogue.exit().await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    log4rs::init_file(
        dotenv::var("LOG_FILE").unwrap(),
        Default::default(),
    )
    .expect("Ошибка инициализации логгера");

    log::info!("Запускаю бота GlebusVPN...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
