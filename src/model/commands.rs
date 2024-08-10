use crate::model::restaurant::Restaurant;
use std::sync::Arc;
use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum BotCommand {
    #[command(description = "Help command")]
    Help,
    #[command(description = "Start command")]
    Start,
    #[command(description = "Reset command")]
    Reset,
}

pub(crate) enum MestCheckCommand {
    Check {
        person_number: u8,
        restaurants: Arc<Vec<Arc<Restaurant>>>,
    },
}
