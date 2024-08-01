use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub(crate) enum Command {
    #[command(description = "Help command")]
    Help,
    #[command(description = "Start command")]
    Start,
    #[command(description = "Reset command")]
    Reset,
}