pub struct Messages;

impl Messages {
    pub fn ru() -> RussianMessages {
        RussianMessages
    }

    // pub fn en() -> EnglishMessages { ... }
}

pub struct RussianMessages;

impl RussianMessages {
    pub fn welcome(&self) -> String {
        "👋 Привет! Я помогу вам подключиться к GlebusVPN. 🚀\n\n\
         Введите количество подключаемых устройств (1-5):"
            .to_string()
    }

    pub fn invalid_state(&self) -> String {
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\n\
         Используйте /help для справки. 😊"
            .to_string()
    }

    pub fn select_platform(&self) -> String {
        "🚀 Отлично! Теперь укажите, пожалуйста, платформу каждого устройства.".to_string()
    }

    pub fn select_platform_for_new_device(&self) -> String {
        "➕ Добавление нового устройства\n\nПожалуйста, выберите платформу для нового устройства:"
            .to_string()
    }

    pub fn device_number(&self, number: u8) -> String {
        format!("📱 Устройство #{number}:")
    }

    pub fn excessive_devices(&self) -> String {
        "❌ Максимальное количество устройств — 5. 😔\n\n\
         Если вам нужно больше, обратитесь к администратору @LineGM. Спасибо за понимание! 🙌"
            .to_string()
    }

    pub fn invalid_device_count(&self) -> String {
        "⚠️ Пожалуйста, введите число от 1 до 5. 🚀".to_string()
    }

    pub fn cancel_operation(&self) -> String {
        "❌ Отменяем текущую операцию.".to_string()
    }

    pub fn error(&self, context: &str) -> String {
        format!(
            "⚠️ Ой, кажется, в {} что-то пошло не так. 😕\n\n\
             Попробуйте ещё раз. 🔄\n\n\
             Если это не поможет, то свяжитесь с администратором.",
            context
        )
    }

    pub fn completion(&self) -> String {
        "🎉 Поздравляем! Ваши подключения успешно созданы. ✅".to_string()
    }

    pub fn connection_info(&self, url: &str) -> String {
        format!(
            "`{}`\n\nВставьте эту ссылку в приложение Hiddify, оно есть на всех предложенных платформах",
            url
        )
    }

    pub fn already_connected(&self) -> String {
        "👋 С возвращением! У вас уже есть подключения к GlebusVPN.".to_string()
    }

    pub fn show_connections(&self) -> String {
        "📱 Показать мои подключения".to_string()
    }

    pub fn your_connections(&self) -> String {
        "🔍 Ваши текущие подключения:".to_string()
    }

    pub fn no_active_connections(&self) -> String {
        "Нет активных подключений".to_string()
    }

    pub fn edit_connections(&self) -> String {
        "✏️ Редактировать подключения".to_string()
    }

    pub fn connection_list_header(&self, available_slots: u8) -> String {
        format!(
            "📱 Ваши подключения (доступно устройств: {})",
            available_slots
        )
    }

    pub fn connection_item(&self, number: u8, platform: &str) -> String {
        format!("{}. {}", number, platform)
    }

    pub fn edit_actions(&self) -> String {
        "Выберите действие:".to_string()
    }

    pub fn add_device(&self) -> String {
        "➕ Добавить устройство".to_string()
    }

    pub fn change_platform(&self) -> String {
        "🔄 Изменить платформу".to_string()
    }

    pub fn delete_device(&self) -> String {
        "❌ Удалить устройство".to_string()
    }

    pub fn back(&self) -> String {
        "⬅️ Вернуться".to_string()
    }

    pub fn select_device_to_edit(&self) -> String {
        "Выберите номер устройства:".to_string()
    }

    pub fn select_new_platform(&self) -> String {
        "Выберите новую платформу:".to_string()
    }

    pub fn connection_not_found(&self) -> String {
        "Подключение не найдено.".to_string()
    }

    pub fn invalid_connection_index(&self) -> String {
        "Неверный индекс подключения.".to_string()
    }

    pub fn invalid_platform(&self) -> String {
        "Неверная платформа.".to_string()
    }

    pub fn platform_changed(&self, platform: &str) -> String {
        format!("Платформа изменена на {}.", platform)
    }
}
