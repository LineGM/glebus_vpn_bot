pub struct Messages;

impl Messages {
    pub fn ru() -> RussianMessages {
        RussianMessages
    }
}

pub struct RussianMessages;

impl RussianMessages {
    pub fn welcome_prompt(&self) -> String {
        "👋 Привет! Я помогу вам подключиться к GlebusVPN 🚀".to_string()
    }

    pub fn invalid_input(&self) -> String {
        "⚠️ Ой, кажется, вы ввели что-то непонятное. 😅\n\n\
         Используйте /help для справки. 😊"
            .to_string()
    }

    pub fn error(&self, context: &str) -> String {
        format!(
            "⚠️ Ой, кажется, в {} что-то пошло не так. 😕\n\n\
             Попробуйте ещё раз. 🔄\n\n\
             Если это не поможет, то свяжитесь с администратором.",
            context
        )
    }

    pub fn new_user_confirmed(&self) -> String {
        "🚀 Давай!".to_string()
    }

    pub fn back(&self) -> String {
        "⬅️ Вернуться".to_string()
    }
}
