use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum BotCommand {
    #[command(description = "Начать использование")]
    Start,
    #[command(description = "Сбросить состояние диалога")]
    Reset,
    #[command(description = "Показать список всех команд")]
    Help,
}

pub(crate) enum MestCheckCommand {
    Check {
        person_number: u8,
        longitude: f64,
        latitude: f64,
    },
}
