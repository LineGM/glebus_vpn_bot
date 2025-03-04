use super::error::MyError;
use super::messages::Messages;
use super::types::{Command, HandlerResult, MyDialogue, State};
use super::xui_api::ThreeXUiClient;
use fast_qr::convert::{Builder, Shape, image::ImageBuilder};
use fast_qr::qr::QRBuilder;
use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use uuid::Uuid;

const MAX_DEVICES: u8 = 5;
const SUPPORTED_PLATFORMS: [&str; 5] = ["Windows", "Android", "Linux", "MacOS", "iOS"];

/// Extracts the username from a `Message` or returns "unknown" if there is none.
///
/// # Arguments
///
/// * `msg` - The `Message` to extract the username from.
///
/// # Returns
///
/// The username if one exists, otherwise "unknown".
fn get_username(msg: &Message) -> &str {
    msg.from
        .as_ref()
        .and_then(|user| user.username.as_deref())
        .unwrap_or("unknown")
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
pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let username = get_username(&msg);
    let chat_id = msg.chat.id;

    log::info!("User {} (chat_id={}) called /start", username, chat_id);

    let base_url = dotenv::var("PANEL_BASE_URL")?;
    let mut api = ThreeXUiClient::new(&base_url);

    let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
    let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

    if let Err(e) = api.login(&admin_login, &admin_password).await {
        log::error!("Failed to login to panel: {}", e);
        bot.send_message(chat_id, Messages::ru().error("панели сервера"))
            .await?;
        return Ok(());
    }

    match api.has_existing_client(chat_id.0).await {
        Ok(true) => {
            let keyboard = InlineKeyboardMarkup::new([[
                InlineKeyboardButton::callback(
                    Messages::ru().show_connections(),
                    "show_connections",
                ),
                InlineKeyboardButton::callback(
                    Messages::ru().edit_connections(),
                    "edit_connections",
                ),
            ]]);

            bot.send_message(chat_id, Messages::ru().already_connected())
                .reply_markup(keyboard)
                .await?;
        }
        Ok(false) => {
            bot.send_message(dialogue.chat_id(), Messages::ru().welcome())
                .await?;
            dialogue.update(State::ReceiveDeviceCount).await?;
        }
        Err(e) => {
            log::error!("Failed to check existing client: {}", e);
            bot.send_message(
                chat_id,
                Messages::ru().error("проверке существующих подключений"),
            )
            .await?;
        }
    }

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
    let username = get_username(&msg); // Extract username from the message
    let chat_id = msg.chat.id; // Extract chat ID from the message

    // Log the usage of the /help command with the username and chat ID
    log::info!("User {} (chat_id={}) called /help", username, chat_id);

    // Send a message with the descriptions of all available commands to the user
    bot.send_message(chat_id, Command::descriptions().to_string())
        .await?;

    Ok(())
}

/// Handles the `/cancel` command, terminating the current operation.
///
/// This function logs the cancellation request, sends a message to the user
/// indicating the cancellation of the current operation, and exits the dialogue.
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
pub async fn cancel(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // Extract the username from the message
    let username = get_username(&msg);
    // Extract the chat ID from the message
    let chat_id = msg.chat.id;

    // Log the cancellation request with the username and chat ID
    log::info!("User {} (chat_id={}) called /cancel", username, chat_id);

    // Send a cancellation message to the user
    bot.send_message(chat_id, Messages::ru().cancel_operation())
        .await?;
    // Exit the dialogue
    dialogue.exit().await?;
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
pub async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    // Extract the username from the message
    let username = get_username(&msg);
    // Extract the chat ID from the message
    let chat_id = msg.chat.id;
    // Extract the user's input from the message
    let user_input = msg.text().unwrap_or_default();

    // Log the incorrect user input
    log::warn!(
        "User {} (chat_id={}) entered an incorrect value: {}",
        username,
        chat_id,
        user_input
    );

    // Send a message to the user indicating that the input was incorrect
    bot.send_message(chat_id, Messages::ru().invalid_state())
        .await?;
    Ok(())
}

/// Handles the device count input from the user.
///
/// This function parses the user's input, checks if it is a valid number of devices.
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
pub async fn receive_device_count(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    // Extract the user's input from the message
    let user_input = msg.text().unwrap_or_default();
    // Extract the username from the message
    let username = get_username(&msg);
    // Extract the chat ID from the message
    let chat_id = msg.chat.id;

    // Parse the user's input as an unsigned 8-bit integer
    match user_input.parse::<u8>() {
        // If the input is a valid number of devices, call handle_valid_device_count
        Ok(count) if (1..=MAX_DEVICES).contains(&count) => {
            handle_valid_device_count(bot, dialogue, username, chat_id, count).await
        }
        // If the input is a number greater than the maximum number of devices, call handle_excessive_device_count
        Ok(count) if count > MAX_DEVICES => {
            handle_excessive_device_count(bot, username, chat_id, count).await
        }
        // If the input is invalid, call handle_invalid_device_count
        _ => handle_invalid_device_count(bot, username, chat_id, user_input).await,
    }
}

/// Handles a valid device count input from the user.
///
/// This function logs the user's input, sends a message to the user to ask for
/// the platform of each device, and updates the dialogue state to
/// `ReceiveDeviceInfo` with the given device count and an empty list of applications.
///
/// # Arguments
///
/// * `bot` - The bot handle.
/// * `dialogue` - The dialogue handle.
/// * `username` - The username of the user.
/// * `chat_id` - The chat ID of the user.
/// * `count` - The number of devices to setup.
///
/// # Returns
///
/// A `HandlerResult`.
async fn handle_valid_device_count(
    bot: Bot,
    dialogue: MyDialogue,
    username: &str,
    chat_id: ChatId,
    count: u8,
) -> HandlerResult {
    // Log the user's input
    log::info!(
        "User {} (chat_id={}) started VPN setup for {} devices",
        username,
        dialogue.chat_id(),
        count
    );

    // Send a message to the user asking for the platform of each device
    bot.send_message(chat_id, Messages::ru().select_platform())
        .await?;

    // Update the dialogue state to ReceiveDeviceInfo with the given device count
    // and an empty list of applications
    dialogue
        .update(State::ReceiveDeviceInfo {
            total_devices: count,
            current_device: 1,
            applications: Vec::new(),
        })
        .await?;

    // Ask the user to select a platform for the first device
    ask_device_platform(&bot, chat_id, 1).await
}

/// Handles an excessive amount of devices specified by the user.
///
/// This function logs a warning when the user enters a device count that exceeds
/// the maximum allowed limit of 5.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages to the user.
/// * `username` - The username of the user who entered the excessive device count.
/// * `chat_id` - The chat ID of the user.
/// * `count` - The number of devices specified by the user, which exceeds the allowed limit.
///
/// # Returns
///
/// A `HandlerResult`.
async fn handle_excessive_device_count(
    bot: Bot,
    username: &str,
    chat_id: ChatId,
    count: u8,
) -> HandlerResult {
    // Log a warning about the excessive device count input
    log::warn!(
        "User {} (chat_id={}) entered an excessive amount of devices: {}",
        username,
        chat_id,
        count
    );

    // Send a message to the user
    bot.send_message(chat_id, Messages::ru().excessive_devices())
        .await?;

    Ok(())
}

/// Handles an invalid device count input from the user.
///
/// This function logs a warning when the user enters an incorrect number of devices,
/// and sends a message to the user prompting them to enter a valid number between 1 and 5.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages to the user.
/// * `username` - The username of the user who entered the invalid device count.
/// * `chat_id` - The chat ID of the user who entered the invalid device count.
/// * `user_input` - The user's input, which is expected to be an invalid number of devices.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message sending operation.
async fn handle_invalid_device_count(
    bot: Bot,
    username: &str,
    chat_id: ChatId,
    user_input: &str,
) -> HandlerResult {
    // Log a warning about the invalid device count input
    log::warn!(
        "User {} (chat_id={}) entered an incorrect amount of devices: {}",
        username,
        chat_id,
        user_input
    );

    // Send a message to the user
    bot.send_message(chat_id, Messages::ru().invalid_device_count())
        .await?;

    // Return a successful HandlerResult
    Ok(())
}

/// Asks the user to select a platform for a specified device number.
///
/// This function sends a message with an inline keyboard containing buttons
/// for each supported platform, allowing the user to choose a platform for
/// the specified device.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages.
/// * `chat_id` - The chat ID of the user to whom the message will be sent.
/// * `device_num` - The number of the device for which the platform is being selected.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message sending operation.
async fn ask_device_platform(bot: &Bot, chat_id: ChatId, device_num: u8) -> HandlerResult {
    // Create the inline keyboard buttons for each supported platform
    let platforms = SUPPORTED_PLATFORMS
        .iter()
        .map(|&p| InlineKeyboardButton::callback(p, p)) // Create a callback button for each platform
        .collect::<Vec<_>>();

    // Send the message with the inline keyboard to the user
    bot.send_message(chat_id, Messages::ru().device_number(device_num))
        .reply_markup(InlineKeyboardMarkup::new([platforms])) // Attach the keyboard markup
        .await?;

    // Return a successful HandlerResult
    Ok(())
}

/// Handles a callback query from a user that selected a platform for a device
/// from the inline keyboard.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages.
/// * `dialogue` - The dialogue handle used to retrieve the chat ID and update the dialogue state.
/// * `(total_devices, current_device, mut applications)` - A tuple containing:
///     + `total_devices`: The total number of devices to setup.
///     + `current_device`: The current device number to setup.
///     + `mut applications`: A mutable vector of strings containing the selected
///       platforms for each device up to `current_device - 1`.
/// * `q` - The callback query containing the selected platform.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message sending
/// and API operations.
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

    // Attempt to add a client to the panel using the selected platform
    if !handle_api_operations(&bot, &dialogue, username, platform).await? {
        return Ok(());
    }

    // If we have setup all devices, send a completion message to the user
    if current_device < total_devices {
        handle_next_device(&bot, dialogue, total_devices, current_device, applications).await?;
    } else {
        handle_completion(&bot, dialogue, username).await?;
    }

    Ok(())
}

/// Handles API operations for adding a client to the panel.
///
/// This function performs the following operations:
/// - Initializes the `ThreeXUiClient` with the base URL from environment variables.
/// - Attempts to log in to the panel using the admin credentials.
/// - If login is unsuccessful, sends an error message and exits the dialogue.
/// - Attempts to add a client with the specified username and platform.
/// - If adding the client is unsuccessful, sends an error message and exits the dialogue.
/// - Sends connection information to the user if all operations are successful.
///
/// # Arguments
///
/// * `bot` - The bot handle.
/// * `dialogue` - The dialogue handle.
/// * `username` - The username of the user.
/// * `platform` - The platform selected by the user.
///
/// # Returns
///
/// A `Result` containing `true` if all operations are successful, or `false` if any operation fails.
pub async fn handle_api_operations(
    bot: &Bot,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> Result<bool, MyError> {
    // Initialize the ThreeXUiClient with the base URL from environment variables
    let base_url = dotenv::var("PANEL_BASE_URL")?;
    let mut api = ThreeXUiClient::new(&base_url);

    // Attempt to log in to the panel using the admin credentials
    let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
    let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

    if !try_login(&mut api, &admin_login, &admin_password).await {
        // If login is unsuccessful, send an error message and exit the dialogue
        send_error_message(bot, dialogue, "панели сервера").await?;
        dialogue.exit().await?;
        return Ok(false);
    }

    // Attempt to add a client with the specified username and platform
    if !try_add_client(&mut api, dialogue, username, platform).await {
        // If adding the client is unsuccessful, send an error message and exit the dialogue
        send_error_message(bot, dialogue, "добавлении очередного подключения").await?;
        dialogue.exit().await?;
        return Ok(false);
    }

    // Send connection information to the user if all operations are successful
    send_connection_info(bot, dialogue, username, platform).await?;
    Ok(true)
}

/// Tries to log in to the panel using the provided username and password.
///
/// # Arguments
///
/// * `api` - The API client to use for the login operation.
/// * `login` - The username to use for the login operation.
/// * `password` - The password to use for the login operation.
///
/// # Returns
///
/// A boolean indicating whether the login operation was successful.
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

/// Tries to add a new client to the panel with the provided username and platform.
///
/// This function constructs a client ID based on the username and platform,
/// creates a new client JSON object, and attempts to add the client using the
/// provided API client.
///
/// # Arguments
///
/// * `api` - The API client to use for the add client operation.
/// * `dialogue` - The dialogue to use for logging and retrieving chat ID.
/// * `username` - The username to use for the client ID.
/// * `platform` - The platform to use for the client ID.
///
/// # Returns
///
/// A boolean indicating whether the add client operation was successful.
async fn try_add_client(
    api: &mut ThreeXUiClient,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> bool {
    // Construct the client ID by combining the username and platform
    // Generate a unique client ID using UUID
    let client_id = format!(
        "{}_{}_{}",
        username,
        platform.to_lowercase(),
        Uuid::new_v4().simple()
    );

    // Create a new client JSON object with the necessary details
    let new_client = serde_json::json!({
        "clients": [{
            "id": Uuid::new_v4().simple().to_string(), // Generate a unique ID for the client
            "email": client_id, // Use the constructed client ID as the email
            "comment": "Added through GlebusVPN bot.", // Add a comment for the client
            "flow": "xtls-rprx-vision", // Specify the flow type
            "enable": true, // Enable the client
            "tgId": dialogue.chat_id(), // Retrieve the chat ID from the dialogue
            "subId": client_id // Use the constructed client ID as the subscription ID
        }]
    });

    // Attempt to add the client using the API client
    match api.add_client(1, &new_client).await {
        Ok(json) => {
            log::info!("Add client result: {}", json); // Log success
            true // Return true on success
        }
        Err(json) => {
            log::error!("Add client result: {}", json); // Log failure
            false // Return false on failure
        }
    }
}

/// Sends an error message to the user.
///
/// This function sends a formatted error message to the user, indicating that
/// something went wrong in the specified context. It suggests the user to try
/// again and to contact the administrator if the issue persists.
///
/// # Arguments
///
/// * `bot` - The bot handle that will send the message.
/// * `dialogue` - The dialogue handle used to retrieve the chat ID.
/// * `error_context` - A string slice indicating the context in which the error
///   occurred. This can be the name of the panel, a description of the operation
///   that failed, or any other relevant context.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message sending
/// operation.
async fn send_error_message(
    bot: &Bot,
    dialogue: &MyDialogue,
    error_context: &str,
) -> HandlerResult {
    // Send the message to the user
    bot.send_message(dialogue.chat_id(), Messages::ru().error(error_context))
        .await?;

    // Return a successful HandlerResult
    Ok(())
}

/// Sends the connection information to the user.
///
/// This function generates a QR code, saves it to a temporary file,
/// sends the QR code to the user as a photo, and removes the temporary file.
/// It also sends the connection URL to the user, which can be used to connect to the VPN.
///
/// # Arguments
///
/// * `bot` - The bot handle that will send the message and the photo.
/// * `dialogue` - The dialogue handle used to retrieve the chat ID.
/// * `username` - The username of the client.
/// * `platform` - The platform of the client.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message and photo
/// sending operations.
async fn send_connection_info(
    bot: &Bot,
    dialogue: &MyDialogue,
    username: &str,
    platform: &str,
) -> HandlerResult {
    // Generate a client ID from the username and platform
    // Generate a unique client ID using UUID
    let client_id = format!(
        "{}_{}_{}",
        username,
        platform.to_lowercase(),
        Uuid::new_v4().simple()
    );

    // Construct the URL of the client's configuration file
    let sub_url = format!("{}/{}", dotenv::var("SUB_BASE_URL")?, client_id);

    // Create a temporary directory to store the QR code image
    let temp_dir = std::env::temp_dir();

    // Generate the name of the temporary file
    let image_name = temp_dir.join(format!("{}.png", client_id));

    // Generate the QR code and save it to the temporary file
    let qrcode = QRBuilder::new(sub_url.clone()).build()?;
    ImageBuilder::default()
        .shape(Shape::RoundedSquare)
        .background_color([255, 255, 255, 0])
        .fit_width(600)
        .to_file(
            &qrcode,
            image_name
                .to_str()
                .ok_or(MyError::Str("Invalid path encoding".to_string()))?,
        )?;

    // Send the QR code to the user as a photo
    bot.send_photo(
        dialogue.chat_id(),
        teloxide::types::InputFile::file(&image_name),
    )
    .await?;

    // Remove the temporary QR code file
    if let Err(e) = std::fs::remove_file(&image_name) {
        log::warn!("Failed to remove temporary QR code file: {}", e);
    }

    // Send the connection information string to the user
    bot.send_message(dialogue.chat_id(), Messages::ru().connection_info(&sub_url))
        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
        .await?;

    Ok(())
}

/// Proceeds to handle the next device in the setup process.
///
/// This function increments the current device counter, updates the dialogue state with the new
/// device number, and prompts the user to select a platform for the next device.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages to the user.
/// * `dialogue` - The dialogue handle used to update the dialogue state.
/// * `total_devices` - The total number of devices to setup.
/// * `current_device` - The current device number being processed.
/// * `applications` - A vector of strings containing the selected platforms for each device.
///
/// # Returns
///
/// A `HandlerResult`.
async fn handle_next_device(
    bot: &Bot,
    dialogue: MyDialogue,
    total_devices: u8,
    current_device: u8,
    applications: Vec<String>,
) -> HandlerResult {
    // Increment the current device counter
    let next_device = current_device + 1;
    // Update the dialogue state with the new device number
    dialogue
        .update(State::ReceiveDeviceInfo {
            total_devices,
            current_device: next_device,
            applications,
        })
        .await?;
    // Prompt the user to select a platform for the next device
    ask_device_platform(bot, dialogue.chat_id(), next_device).await
}

/// Handles the completion of the VPN setup process.
///
/// This function sends a completion message to the user, logs the completion event,
/// and exits the dialogue, marking the end of the VPN setup process.
///
/// # Arguments
///
/// * `bot` - The bot handle used to send messages to the user.
/// * `dialogue` - The dialogue handle used to manage the user's dialogue state.
/// * `username` - The username of the user who completed the VPN setup.
/// * `q` - The callback query associated with the completion.
///
/// # Returns
///
/// A `HandlerResult` indicating the success or failure of the message sending operation.
async fn handle_completion(bot: &Bot, dialogue: MyDialogue, username: &str) -> HandlerResult {
    // Log the successful completion of the VPN setup for the user
    log::info!(
        "User {} (chat_id={}) successfully completed the request",
        username,
        dialogue.chat_id()
    );

    // Send a completion message to the user
    bot.send_message(dialogue.chat_id(), Messages::ru().completion())
        .await?;

    // Exit the dialogue as the process is complete
    dialogue.exit().await?;

    // Return a successful HandlerResult
    Ok(())
}

pub async fn show_connections(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;
        let base_url = dotenv::var("PANEL_BASE_URL")?;
        let mut api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

        if let Err(e) = api.login(&admin_login, &admin_password).await {
            log::error!("Failed to login to panel: {}", e);
            bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                .await?;
            return Ok(());
        }

        match api.get_client_connections(chat_id.0).await {
            Ok(connections_str) => {
                let sub_base_url = dotenv::var("SUB_BASE_URL")?;

                if let Ok(connections) =
                    serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                {
                    bot.send_message(chat_id, Messages::ru().your_connections())
                        .await?;

                    let mut formatted_connections = String::new();

                    for connection in connections {
                        if let (Some(email), Some(sub_id)) = (
                            connection.get("email").and_then(|e| e.as_str()),
                            connection.get("subId").and_then(|s| s.as_str()),
                        ) {
                            let sub_url = format!("{}/{}", sub_base_url, sub_id);
                            formatted_connections.push_str(&format!(
                                "*{}*: `{}`\n\n",
                                email.replace("_", " \\| "),
                                sub_url
                            ));
                        }
                    }

                    let keyboard = InlineKeyboardMarkup::new([[InlineKeyboardButton::callback(
                        Messages::ru().back(),
                        "back_to_main_menu",
                    )]]);

                    if !formatted_connections.is_empty() {
                        bot.send_message(chat_id, format!("{}", formatted_connections))
                            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                            .reply_markup(keyboard)
                            .await?;
                    } else {
                        bot.send_message(chat_id, Messages::ru().no_active_connections())
                            .reply_markup(keyboard)
                            .await?;
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to get client connections: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о подключениях"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn edit_connections(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;

        let keyboard = InlineKeyboardMarkup::new([
            [InlineKeyboardButton::callback(
                Messages::ru().change_platform(),
                "change_current_connections",
            )],
            [InlineKeyboardButton::callback(
                Messages::ru().add_device(),
                "add_connection",
            )],
            [InlineKeyboardButton::callback(
                Messages::ru().delete_device(),
                "delete_connections",
            )],
            [InlineKeyboardButton::callback(
                Messages::ru().back(),
                "back_to_main_menu",
            )],
        ]);

        bot.send_message(chat_id, Messages::ru().edit_actions())
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}

pub async fn start_callback(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;
        let base_url = dotenv::var("PANEL_BASE_URL")?;
        let mut api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

        if let Err(e) = api.login(&admin_login, &admin_password).await {
            log::error!("Failed to login to panel: {}", e);
            bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                .await?;
            return Ok(());
        }

        match api.has_existing_client(chat_id.0).await {
            Ok(true) => {
                let keyboard = InlineKeyboardMarkup::new([[
                    InlineKeyboardButton::callback(
                        Messages::ru().show_connections(),
                        "show_connections",
                    ),
                    InlineKeyboardButton::callback(
                        Messages::ru().edit_connections(),
                        "edit_connections",
                    ),
                ]]);

                bot.send_message(chat_id, Messages::ru().already_connected())
                    .reply_markup(keyboard)
                    .await?;
            }
            Ok(false) => {
                bot.send_message(chat_id, Messages::ru().welcome()).await?;
                // dialogue.update(State::ReceiveDeviceCount).await?; // Убрали обновление состояния, так как это новый запрос
            }
            Err(e) => {
                log::error!("Failed to check existing client: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("проверке наличия подключений"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn edit_connection(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;

        // Извлекаем индекс подключения из callback_data
        let connection_index = q
            .data
            .as_ref()
            .and_then(|data| data.split('_').last())
            .and_then(|index| index.parse::<usize>().ok());

        match connection_index {
            Some(index) => {
                // Получаем информацию о подключениях пользователя из панели управления
                let base_url = dotenv::var("PANEL_BASE_URL")?;
                let mut api = ThreeXUiClient::new(&base_url);

                let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
                let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

                if let Err(e) = api.login(&admin_login, &admin_password).await {
                    log::error!("Failed to login to panel: {}", e);
                    bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                        .await?;
                    return Ok(());
                }

                match api.get_client_connections(chat_id.0).await {
                    Ok(connections_str) => {
                        if let Ok(connections) =
                            serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                        {
                            if let Some(connection) = connections.get(index) {
                                // Предлагаем пользователю выбрать новую платформу
                                let keyboard = SUPPORTED_PLATFORMS
                                    .iter()
                                    .map(|&p| {
                                        InlineKeyboardButton::callback(
                                            p,
                                            format!("change_platform_{}_{}", index, p),
                                        )
                                    })
                                    .collect::<Vec<_>>();

                                let reply_markup = InlineKeyboardMarkup::new([keyboard]);

                                bot.send_message(chat_id, Messages::ru().select_new_platform())
                                    .reply_markup(reply_markup)
                                    .await?;
                            } else {
                                bot.send_message(chat_id, Messages::ru().connection_not_found())
                                    .await?;
                            }
                        } else {
                            log::error!("Failed to parse client connections");
                            bot.send_message(
                                chat_id,
                                Messages::ru().error("получении информации о подключениях"),
                            )
                            .await?;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get client connections: {}", e);
                        bot.send_message(
                            chat_id,
                            Messages::ru().error("получении информации о подключениях"),
                        )
                        .await?;
                    }
                }
            }
            None => {
                bot.send_message(chat_id, Messages::ru().invalid_connection_index())
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn change_platform(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.clone().message {
        let chat_id = msg.chat().id;

        // Извлекаем индекс подключения и новую платформу из callback_data
        let parts: Vec<&str> = q.data.as_ref().unwrap().split('_').collect();
        let connection_index = parts.get(2).and_then(|index| index.parse::<usize>().ok());
        let new_platform = parts.get(3).map(|&platform| platform.to_string());

        match (connection_index, new_platform) {
            (Some(index), Some(platform)) => {
                // Получаем информацию о подключениях пользователя из панели управления
                let base_url = dotenv::var("PANEL_BASE_URL")?;
                let mut api = ThreeXUiClient::new(&base_url);

                let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
                let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

                if let Err(e) = api.login(&admin_login, &admin_password).await {
                    log::error!("Failed to login to panel: {}", e);
                    bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                        .await?;
                    return Ok(());
                }

                match api.get_client_connections(chat_id.0).await {
                    Ok(connections_str) => {
                        if let Ok(mut connections) =
                            serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                        {
                            if let Some(connection) = connections.get_mut(index) {
                                // Обновляем email и subId
                                if let (Some(email), Some(sub_id)) = (
                                    connection.get("email").and_then(|e| e.as_str()),
                                    connection.get("subId").and_then(|s| s.as_str()),
                                ) {
                                    // Извлекаем имя пользователя из email
                                    let new_username = email.split('_').next().unwrap_or("unknown");
                                    let new_platform = platform.to_lowercase();

                                    // Формируем новые email и subId
                                    let new_email = format!("{}_{}", new_username, new_platform);
                                    let new_sub_id = format!("{}_{}", new_username, new_platform);

                                    // Обновляем значения в connection
                                    let mut connection_clone = connection.clone();
                                    if let Some(connection_map) = connection_clone.as_object_mut() {
                                        connection_map
                                            .insert("email".to_string(), new_email.clone().into());
                                        connection_map
                                            .insert("subId".to_string(), new_sub_id.clone().into());
                                    }

                                    // Обновляем подключение в панели управления
                                    let client_id = connection
                                        .get("id")
                                        .and_then(|id| id.as_str())
                                        .unwrap_or_default()
                                        .to_string();

                                    // Получаем inbound_id из connection
                                    let inbound_id = connection
                                        .get("id")
                                        .and_then(|id| id.as_i64())
                                        .unwrap_or(1)
                                        as u32;

                                    // Удаляем существующего клиента
                                    let delete_result =
                                        api.delete_client(inbound_id, &client_id).await;

                                    match delete_result {
                                        Ok(_) => {
                                            log::info!(
                                                "Update client: Old client deleted successfully"
                                            );

                                            // Добавляем нового клиента с помощью try_add_client
                                            if !try_add_client(
                                                &mut api,
                                                &dialogue,
                                                &new_username,
                                                &new_platform,
                                            )
                                            .await
                                            {
                                                log::error!(
                                                    "Update client: Failed to add new client"
                                                );
                                                bot.send_message(
                                                    chat_id,
                                                    Messages::ru().error("добавлении подключения"),
                                                )
                                                .await?;
                                            } else {
                                                log::info!(
                                                    "Update client: New client added successfully"
                                                );
                                                bot.send_message(
                                                    chat_id,
                                                    Messages::ru().platform_changed(&platform),
                                                )
                                                .await?;
                                                edit_connections(bot, q).await?;
                                            }
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "Update client: Failed to delete old client: {}",
                                                e
                                            );
                                            bot.send_message(
                                                chat_id,
                                                Messages::ru().error("удалении подключения"),
                                            )
                                            .await?;
                                        }
                                    }
                                }
                            } else {
                                bot.send_message(chat_id, Messages::ru().connection_not_found())
                                    .await?;
                            }
                        } else {
                            log::error!("Update client: Failed to parse client connections");
                            bot.send_message(
                                chat_id,
                                Messages::ru().error("получении информации о подключениях"),
                            )
                            .await?;
                        }
                    }
                    Err(e) => {
                        log::error!("Update client: Failed to get client connections: {}", e);
                        bot.send_message(
                            chat_id,
                            Messages::ru().error("получении информации о подключениях"),
                        )
                        .await?;
                    }
                }
            }
            _ => {
                bot.send_message(chat_id, Messages::ru().invalid_platform())
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn change_current_connections(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;

        // Получаем информацию о подключениях пользователя из панели управления
        let base_url = dotenv::var("PANEL_BASE_URL")?;
        let mut api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

        if let Err(e) = api.login(&admin_login, &admin_password).await {
            log::error!("Failed to login to panel: {}", e);
            bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                .await?;
            return Ok(());
        }

        match api.get_client_connections(chat_id.0).await {
            Ok(connections_str) => {
                if let Ok(connections) =
                    serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                {
                    // Проверяем, есть ли подключения
                    if connections.is_empty() {
                        bot.send_message(chat_id, Messages::ru().no_active_connections())
                            .await?;
                        return Ok(());
                    }

                    // Отправляем пользователю список подключений с кнопками для редактирования
                    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                    for (i, connection) in connections.iter().enumerate() {
                        if let Some(email) = connection.get("email").and_then(|e| e.as_str()) {
                            let button_text = Messages::ru().connection_item((i + 1) as u8, email);
                            let callback_data = format!("edit_connection_{}", i);
                            keyboard.push(vec![InlineKeyboardButton::callback(
                                button_text,
                                callback_data,
                            )]);
                        }
                    }

                    // Добавляем кнопку "Назад"
                    keyboard.push(vec![InlineKeyboardButton::callback(
                        Messages::ru().back(),
                        "back_to_edit_menu",
                    )]);

                    let reply_markup = InlineKeyboardMarkup::new(keyboard);

                    bot.send_message(chat_id, Messages::ru().select_device_to_edit())
                        .reply_markup(reply_markup)
                        .await?;
                } else {
                    log::error!("Failed to parse client connections");
                    bot.send_message(
                        chat_id,
                        Messages::ru().error("получении информации о подключениях"),
                    )
                    .await?;
                }
            }
            Err(e) => {
                log::error!("Failed to get client connections: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о подключениях"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn add_connection(bot: Bot, dialogue: MyDialogue, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.clone().message {
        let chat_id = msg.chat().id;

        // Получаем информацию о подключениях пользователя из панели управления
        let base_url = dotenv::var("PANEL_BASE_URL")?;
        let mut api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

        if let Err(e) = api.login(&admin_login, &admin_password).await {
            log::error!("Failed to login to panel: {}", e);
            bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                .await?;
            return Ok(());
        }

        match api.get_client_connections(chat_id.0).await {
            Ok(connections_str) => {
                if let Ok(connections) =
                    serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                {
                    // Проверяем, не превышен ли лимит устройств
                    if connections.len() >= MAX_DEVICES.into() {
                        bot.send_message(chat_id, Messages::ru().excessive_devices())
                            .await?;
                        edit_connections(bot, q).await?;
                        return Ok(());
                    }

                    // Запрашиваем у пользователя платформу для нового устройства
                    dialogue
                        .update(State::ReceiveDeviceInfo {
                            total_devices: 1,
                            current_device: 1,
                            applications: Vec::new(),
                        })
                        .await?;

                    bot.send_message(chat_id, Messages::ru().select_platform_for_new_device())
                        .await?;
                    ask_device_platform(&bot, chat_id, 1).await?;
                } else {
                    log::error!("Failed to parse client connections");
                    bot.send_message(
                        chat_id,
                        Messages::ru().error("получении информации о подключениях"),
                    )
                    .await?;
                }
            }
            Err(e) => {
                log::error!("Failed to get client connections: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о подключениях"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn delete_connections(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;

        // Получаем информацию о подключениях пользователя из панели управления
        let base_url = dotenv::var("PANEL_BASE_URL")?;
        let mut api = ThreeXUiClient::new(&base_url);

        let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
        let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

        if let Err(e) = api.login(&admin_login, &admin_password).await {
            log::error!("Failed to login to panel: {}", e);
            bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                .await?;
            return Ok(());
        }

        match api.get_client_connections(chat_id.0).await {
            Ok(connections_str) => {
                if let Ok(connections) =
                    serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                {
                    // Проверяем, есть ли подключения
                    if connections.is_empty() {
                        bot.send_message(chat_id, Messages::ru().no_active_connections())
                            .await?;
                        return Ok(());
                    }

                    // Отправляем пользователю список подключений с кнопками для удаления
                    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                    for (i, connection) in connections.iter().enumerate() {
                        if let Some(email) = connection.get("email").and_then(|e| e.as_str()) {
                            let button_text = Messages::ru().connection_item((i + 1) as u8, email);
                            let callback_data = format!("delete_connection_{}", i);
                            keyboard.push(vec![InlineKeyboardButton::callback(
                                button_text,
                                callback_data,
                            )]);
                        }
                    }

                    // Добавляем кнопку "Назад"
                    keyboard.push(vec![InlineKeyboardButton::callback(
                        Messages::ru().back(),
                        "back_to_edit_menu",
                    )]);

                    let reply_markup = InlineKeyboardMarkup::new(keyboard);

                    bot.send_message(chat_id, "Выберите подключение для удаления:")
                        .reply_markup(reply_markup)
                        .await?;
                } else {
                    log::error!("Failed to parse client connections");
                    bot.send_message(
                        chat_id,
                        Messages::ru().error("получении информации о подключениях"),
                    )
                    .await?;
                }
            }
            Err(e) => {
                log::error!("Failed to get client connections: {}", e);
                bot.send_message(
                    chat_id,
                    Messages::ru().error("получении информации о подключениях"),
                )
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn delete_connection(bot: Bot, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.clone().message {
        let chat_id = msg.chat().id;

        // Извлекаем индекс подключения из callback_data
        let connection_index = q
            .data
            .as_ref()
            .and_then(|data| data.split('_').last())
            .and_then(|index| index.parse::<usize>().ok());

        match connection_index {
            Some(index) => {
                // Получаем информацию о подключениях пользователя из панели управления
                let base_url = dotenv::var("PANEL_BASE_URL")?;
                let mut api = ThreeXUiClient::new(&base_url);

                let admin_login = dotenv::var("PANEL_ADMIN_LOGIN")?;
                let admin_password = dotenv::var("PANEL_ADMIN_PASSWORD")?;

                if let Err(e) = api.login(&admin_login, &admin_password).await {
                    log::error!("Failed to login to panel: {}", e);
                    bot.send_message(chat_id, Messages::ru().error("панели сервера"))
                        .await?;
                    return Ok(());
                }

                match api.get_client_connections(chat_id.0).await {
                    Ok(connections_str) => {
                        if let Ok(connections) =
                            serde_json::from_str::<Vec<serde_json::Value>>(&connections_str)
                        {
                            if let Some(connection) = connections.get(index) {
                                // Получаем client_id и inbound_id из connection
                                let client_id = connection
                                    .get("id")
                                    .and_then(|id| id.as_str())
                                    .unwrap_or_default()
                                    .to_string();

                                let inbound_id =
                                    connection.get("id").and_then(|id| id.as_i64()).unwrap_or(1)
                                        as u32;

                                // Удаляем подключение из панели управления
                                let delete_result = api.delete_client(inbound_id, &client_id).await;

                                match delete_result {
                                    Ok(_) => {
                                        log::info!("Client deleted successfully");
                                        bot.send_message(chat_id, "Подключение успешно удалено.")
                                            .await?;

                                        // Возвращаемся в меню редактирования подключений
                                        edit_connections(bot, q).await?;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to delete client: {}", e);
                                        bot.send_message(
                                            chat_id,
                                            Messages::ru().error("удалении подключения"),
                                        )
                                        .await?;
                                    }
                                }
                            } else {
                                bot.send_message(chat_id, Messages::ru().connection_not_found())
                                    .await?;
                            }
                        } else {
                            log::error!("Failed to parse client connections");
                            bot.send_message(
                                chat_id,
                                Messages::ru().error("получении информации о подключениях"),
                            )
                            .await?;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get client connections: {}", e);
                        bot.send_message(
                            chat_id,
                            Messages::ru().error("получении информации о подключениях"),
                        )
                        .await?;
                    }
                }
            }
            None => {
                bot.send_message(chat_id, Messages::ru().invalid_connection_index())
                    .await?;
            }
        }
    }

    Ok(())
}
