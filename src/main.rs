mod background_processing;
mod db;
mod model;
mod schema;
mod utils;

use crate::background_processing::tasks::send_mest_check_notification;
use crate::db::DatabaseHandler;
use crate::model::commands::{BotCommand, MestCheckCommand};
use anyhow::Result;
use dotenv::dotenv;
use model::types::*;
use model::{booking_info::BookingInfo, restaurant, state::State};

use schema::schema;

use std::sync::Arc;

use teloxide::types::MenuButton;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, utils::command::BotCommands};

use tokio::sync::mpsc;

use utils::deserializer::deserialize_restaurants;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pretty_env_logger::init();
    let db = DatabaseHandler::from_env().await;
    log::info!("Starting Mest Net bot...");

    let restaurants = Arc::new(deserialize_restaurants("restaurant_list.json").unwrap());

    let (tx, rx) = mpsc::channel::<MestCheckCommand>(32);

    let restaurants_booking_info: Db<u64, BookingInfo> = Arc::new(scc::HashMap::new());
    let restaurant_by_token: Db<String, u64> = Arc::new(scc::HashMap::new());
    let restaurant_managers: Db<u64, UserId> = Arc::new(scc::HashMap::new());
    let managers_restaurant: Db<UserId, u64> = Arc::new(scc::HashMap::new());

    for restaurant in &*restaurants {
        let _ = restaurants_booking_info.insert(restaurant.id, BookingInfo::new());
        let _ = restaurant_by_token.insert(restaurant.token.clone(), restaurant.id);
    }

    let bot = Bot::from_env();

    bot.set_my_commands(BotCommand::bot_commands()).await?;
    bot.set_chat_menu_button()
        .menu_button(MenuButton::Commands)
        .await?;

    {
        let bot = bot.clone();
        let restaurants_booking_info = restaurants_booking_info.clone();
        let restaurant_managers = restaurant_managers.clone();
        tokio::spawn(async move {
            send_mest_check_notification(bot, rx, restaurants_booking_info, restaurant_managers)
                .await
        });
    }

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            db.clone(),
            InMemStorage::<State>::new(),
            restaurants.clone(),
            restaurants_booking_info.clone(),
            restaurant_by_token.clone(),
            restaurant_managers.clone(),
            managers_restaurant.clone(),
            tx.clone()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
