mod schema;
mod model;
mod utils;

use std::sync::Arc;
use dotenv::dotenv;
use teloxide::{
    dispatching::dialogue::InMemStorage,
    prelude::*,
    utils::command::BotCommands,
};
use teloxide::types::MenuButton;
use schema::schema;
use model::{commands::Command, restaurant, state::State};
use utils::deserializer::deserialize_restaurants;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let restaurants = Arc::new(deserialize_restaurants("restaurant_list.json").unwrap());

    let bot = Bot::from_env();

    Command::descriptions();

    let _ = bot.set_my_commands(Command::bot_commands()).await;
    let _ = bot.set_chat_menu_button().menu_button(MenuButton::Commands).await;

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), restaurants])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}