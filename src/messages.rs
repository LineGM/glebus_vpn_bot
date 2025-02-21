pub struct Messages;

impl Messages {
    pub fn ru() -> RussianMessages {
        RussianMessages
    }

    // В будущем можно добавить другие языки
    // pub fn en() -> EnglishMessages { ... }
}

pub struct RussianMessages;

impl RussianMessages {
    pub fn welcome(&self) -> &'static str {
        "👋 Привет! Я помогу вам подключиться к GlebusVPN. 🚀\n\n\
         Введите количество подключаемых устройств (1-5):"
    }

    pub fn invalid_state(&self) -> &'static str {
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\n\
         Используйте /help для справки. 😊"
    }

    pub fn select_platform(&self) -> &'static str {
        "🚀 Отлично! Теперь укажите, пожалуйста, платформу каждого устройства."
    }

    pub fn device_number(&self, number: u8) -> String {
        format!("📱 Устройство #{number}:")
    }

    pub fn excessive_devices(&self) -> &'static str {
        "❌ Максимальное количество устройств — 5. 😔\n\n\
         Если вам нужно больше, обратитесь к администратору @LineGM. Спасибо за понимание! 🙌"
    }

    pub fn invalid_device_count(&self) -> &'static str {
        "⚠️ Пожалуйста, введите число от 1 до 5. 🚀"
    }

    pub fn cancel_operation(&self) -> &'static str {
        "❌ Отменяем текущую операцию."
    }

    pub fn error(&self, context: &str) -> String {
        format!(
            "⚠️ Ой, кажется, в {} что-то пошло не так. 😕\n\n\
             Попробуйте ещё раз. 🔄\n\n\
             Если это не поможет, то свяжитесь с администратором.",
            context
        )
    }

    pub fn completion(&self) -> &'static str {
        "🎉 Поздравляем! Ваши подключения успешно созданы. ✅"
    }

    pub fn connection_info(&self, url: &str) -> String {
        format!(
            "`{}`\n\nВставьте эту ссылку в приложение Hiddify, оно есть на всех предложенных платформах",
            url
        )
    }
}
