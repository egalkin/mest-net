mod background_processing;
mod db;
mod dialogue_storage;
mod entity;
mod model;
mod schema;
mod utils;

use crate::background_processing::tasks::send_mest_check_notification;
use crate::db::DatabaseHandler;
use crate::model::commands::{BotCommand, MestCheckCommand};
use anyhow::Result;
use dotenv::dotenv;
use model::types::*;
use model::{booking_info::BookingInfo, restaurant::Restaurant, state::State};

use schema::schema;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Bincode;
use teloxide::dispatching::dialogue::{ErasedStorage, Storage};
use teloxide::types::MenuButton;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::mpsc;

use crate::dialogue_storage::skytable_storage::SkytableStorage;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db_handler = DatabaseHandler::from_env().await;

    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    log::info!("Starting Mest Net bot...");

    let restaurants: Arc<Vec<Arc<Restaurant>>> = Arc::new(
        db_handler
            .get_all_restaurants()
            .await
            .iter()
            .map(|restaurant_model| Arc::new(restaurant_model.into()))
            .collect(),
    );
    let restaurants_number = db_handler.count_restaurants().await;
    let (tx, rx) = mpsc::channel::<MestCheckCommand>(32);

    let restaurants_booking_info: Db<i32, BookingInfo> = Arc::new(scc::HashMap::new());

    for restaurant in &*restaurants {
        let _ = restaurants_booking_info
            .insert(restaurant.id, BookingInfo::new(restaurant.name.clone()));
    }

    let skytable_storage: Arc<ErasedStorage<State>> =
        SkytableStorage::open(Bincode).await.unwrap().erase();

    let bot = Bot::from_env();

    bot.set_my_commands(BotCommand::bot_commands()).await?;
    bot.set_chat_menu_button()
        .menu_button(MenuButton::Commands)
        .await?;

    {
        let bot = bot.clone();
        let db_handler = db_handler.clone();
        let restaurants_booking_info = restaurants_booking_info.clone();
        tokio::spawn(async move {
            send_mest_check_notification(bot, rx, db_handler.clone(), restaurants_booking_info)
                .await
        });
    }

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            db_handler.clone(),
            skytable_storage.clone(),
            restaurants.clone(),
            restaurants_booking_info.clone(),
            tx.clone(),
            restaurants_number
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
