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
    ReceivePeopleCount,
    ReceivePlatformChoice {
        people_count: i8,
    },
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Вывести этот текст
    Help,
    /// Начать оформление заявки
    Start,
    /// Отменить процедуру оформления заявки
    Cancel,
}

#[tokio::main]
async fn main() {
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
        .branch(case![State::ReceivePeopleCount].endpoint(receive_people_count))
        .branch(dptree::endpoint(invalid_state));

    let callback_query_handler = Update::filter_callback_query().branch(
        case![State::ReceivePlatformChoice { people_count }].endpoint(receive_platform_selection),
    );

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Давайте начнём!\nЭто бот GlebusVPN и я помогаю регистрировать заявки на подключение.\nСколько людей вы хотите подключить?"
    )
    .await?;
    dialogue.update(State::ReceivePeopleCount).await?;
    Ok(())
}

async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Отмена диалога.").await?;
    dialogue.exit().await?;
    Ok(())
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Невозможно обработать сообщение. Введите /help, чтобы узнать об использовании бота.",
    )
    .await?;
    Ok(())
}

async fn receive_people_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().and_then(|s| s.parse::<i8>().ok()) {
        Some(people_count) => {
            let platforms = ["Windows", "Android", "Linux", "MacOS", "iOS"]
                .map(|platform| InlineKeyboardButton::callback(platform, platform));

            bot.send_message(msg.chat.id, "Выберите платформу:")
                .reply_markup(InlineKeyboardMarkup::new([platforms]))
                .await?;
            dialogue
                .update(State::ReceivePlatformChoice { people_count })
                .await?;
        }
        None => {
            bot.send_message(
                msg.chat.id,
                "Пожалуйста, отправьте мне количество людей для подключения.",
            )
            .await?;
        }
    }
    Ok(())
}

async fn receive_platform_selection(
    bot: Bot,
    dialogue: MyDialogue,
    people_count: i8, // Available from `State::ReceivePlatformChoice`.
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(product) = &q.data {
        bot.send_message(
            dialogue.chat_id(),
            format!("Вы хотите подключить {people_count} людей к GlebusVPN и выбрали платформу '{product}'."),
        )
        .await?;
        dialogue.exit().await?;
    }
    Ok(())
}
