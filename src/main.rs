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
    bot.send_message(
        msg.chat.id,
        "Давайте начнём оформление заявок!\nСколько устройств нужно подключить?",
    )
    .await?;
    dialogue.update(State::ReceiveDeviceCount).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Отмена текущей операции.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Некорректное состояние. Используйте /help для справки.",
    )
    .await?;
    Ok(())
}

async fn receive_device_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().and_then(|s| s.parse::<u8>().ok()) {
        // Добавлена проверка на максимальное количество
        Some(count) if count > 0 && count <= 5 => {
            bot.send_message(msg.chat.id, "Начнём вводить данные для каждого устройства:")
                .await?;
            
            dialogue.update(State::ReceiveDeviceInfo {
                total_devices: count,
                current_device: 1,
                applications: Vec::new(),
            }).await?;
            
            ask_device_platform(&bot, msg.chat.id, 1).await?;
        }
        // Обработка случая с превышением лимита
        Some(count) if count > 5 => {
            bot.send_message(
                msg.chat.id,
                "❌ Максимальное количество устройств - 5.\n\nДля оформления заявки на большее количество устройств, пожалуйста, обратитесь к администратору @LineGM."
            ).await?;
        }
        // Общий случай некорректного ввода
        _ => {
            bot.send_message(
                msg.chat.id,
                "⚠️ Пожалуйста, введите число от 1 до 5!"
            ).await?;
        }
    }
    Ok(())
}

async fn ask_device_platform(bot: &Bot, chat_id: ChatId, device_num: u8) -> HandlerResult {
    let platforms = ["Windows", "Android", "Linux", "MacOS", "iOS"]
        .map(|p| InlineKeyboardButton::callback(p, p));

    bot.send_message(chat_id, format!("Выберите платформу для устройства №{}:", device_num))
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
        applications.push(format!("Устройство {}: {}", current_device, platform));
        
        if current_device < total_devices {
            let next_device = current_device + 1;
            dialogue.update(State::ReceiveDeviceInfo {
                total_devices,
                current_device: next_device,
                applications,
            }).await?;
            
            ask_device_platform(&bot, dialogue.chat_id(), next_device).await?;
        } else {
            let summary = applications.join("\n");
            bot.send_message(
                dialogue.chat_id(),
                format!("✅ Сформировано заявок: {}\n\n{}", total_devices, summary)
            ).await?;
            
            dialogue.exit().await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Запускаю бота GlebusVPN...");

    let bot = Bot::from_env();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}