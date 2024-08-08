use crate::model::commands::BotCommand;
use crate::model::commands::MestCheckCommand;
use crate::model::state::State::Start;
use crate::model::{restaurant::Restaurant, state::State, types::*};
use crate::utils::constants::SEARCH_REQUEST_MESSAGE;
use crate::utils::keyboard::*;
use anyhow::Error;
use async_std::task;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use teloxide::types::ReplyMarkup;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};
use tokio::sync::mpsc::Sender;

pub(crate) fn schema() -> UpdateHandler<Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(
            case![Start]
                .branch(case![BotCommand::Help].endpoint(help))
                .branch(case![BotCommand::Start].endpoint(start))
                .branch(case![BotCommand::Reset].endpoint(reset))
                .branch(dptree::endpoint(invalid_input)),
        )
        .branch(case![BotCommand::Reset])
        .endpoint(reset);
    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::RoleSelection].endpoint(receive_role_selection))
        // Admin flow
        .branch(case![State::ReceiveAdminToken].endpoint(receive_admin_token))
        .branch(case![State::WaitingForRequests].endpoint(receive_booking_request))
        // .branch(case![State::WaitingForRequests].endpoint())
        //  User flow
        .branch(case![State::ReceiveSearchRequest].endpoint(receive_search_request))
        .branch(case![State::ReceivePersonNumber].endpoint(receive_person_number))
        .branch(case![State::ReceiveLocation { person_number }].endpoint(receive_location))
        .branch(dptree::endpoint(invalid_input));

    dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler)
}

async fn invalid_input(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please, send /start.")
        .await?;
    Ok(())
}

/// COMMAND HANDLERS
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .await?;
    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Какая у вас роль?")
        .reply_markup(make_role_keyboard())
        .await?;
    dialogue.update(State::RoleSelection).await?;
    Ok(())
}

async fn reset(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .reply_markup(ReplyMarkup::kb_remove())
        .await?;
    dialogue.exit().await?;
    Ok(())
}

/// STATE HANDLERS

async fn receive_role_selection(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some("Обычный пользователь") => {
            bot.send_message(msg.chat.id, "Добро пожаловать в наш бот!")
                .reply_markup(make_search_keyboard())
                .await?;
            dialogue.update(State::ReceiveSearchRequest).await?;
        }
        Some("Администратор") => {
            bot.send_message(msg.chat.id, "Введите ваш токен")
                .reply_markup(ReplyMarkup::kb_remove())
                .await?;
            dialogue.update(State::ReceiveAdminToken).await?
        }
        _ => {
            bot.send_message(msg.chat.id, "Некорректная роль").await?;
        }
    }

    Ok(())
}

async fn receive_admin_token(
    restaurant_by_token: Db<String, u64>,
    restaurant_managers: Db<u64, UserId>,
    managers_restaurant: Db<UserId, u64>,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(token) => match restaurant_by_token.get_async(token).await {
            Some(entry) => {
                let _ = restaurant_managers
                    .insert_async(*entry.get(), msg.from().unwrap().id)
                    .await;
                let _ = managers_restaurant
                    .insert_async(msg.from().unwrap().id, *entry.get())
                    .await;
                bot.send_message(msg.chat.id, "Ожидайте запросов на бронирование")
                    .await?;
                dialogue.update(State::WaitingForRequests).await?
            }
            _ => {
                bot.send_message(msg.chat.id, "Неверный токен").await?;
            }
        },
        _ => {
            bot.send_message(msg.chat.id, "Отправьте токен").await?;
        }
    }

    Ok(())
}

async fn receive_booking_request(
    restaurants_booking_info: Db<u64, u16>,
    managers_restaurant: Db<UserId, u64>,
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    let reply_to_message = msg.reply_to_message();
    match reply_to_message {
        Some(val) => {
            if let Some(text) = val.text() {
                let tokens = text.split_ascii_whitespace().collect::<Vec<&str>>();
                if let Ok(person_number) = tokens[tokens.len() - 2].parse::<u8>() {
                    match msg.text() {
                        Some(ans) if ans == "Да" || ans == "Нет" => {
                            if let Some(manager_id) =
                                managers_restaurant.get_async(&msg.from().unwrap().id).await
                            {
                                if let Some(mut booking_info) =
                                    restaurants_booking_info.get_async(&manager_id).await
                                {
                                    if ans == "Да" {
                                        *booking_info |= 1 << person_number;
                                    }
                                    *booking_info &= !(1 << (8 + person_number));
                                }
                            }
                        }
                        _ => {
                            bot.send_message(msg.chat.id, "Ответьте Да или Нет").await?;
                        }
                    }
                }
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Отправьте ответ исполоьзуя Reply")
                .await?;
        }
    }
    Ok(())
}

async fn receive_search_request(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(SEARCH_REQUEST_MESSAGE) => {
            bot.send_message(msg.chat.id, "Сколько гостей будет?")
                .reply_markup(make_number_keyboard())
                .await?;
            dialogue.update(State::ReceivePersonNumber).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Некорректная команда")
                .await?;
        }
    }

    Ok(())
}

async fn receive_person_number(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(person_number)) => {
            bot.send_message(msg.chat.id, "Отправьте локацию для поиска мест")
                .reply_markup(make_location_keyboard())
                .await?;
            dialogue
                .update(State::ReceiveLocation { person_number })
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Отправьте число гостей")
                .await?;
        }
    }

    Ok(())
}

async fn receive_location(
    restaurants: Arc<Vec<Restaurant>>,
    restaurants_booking_info: Db<u64, u16>,
    sender: Sender<MestCheckCommand>,
    bot: Bot,
    dialogue: MyDialogue,
    person_number: u8,
    msg: Message,
) -> HandlerResult {
    match msg.location() {
        Some(location) => {
            let closest_restaurants_ids: Vec<u64> = restaurants
                .iter()
                .filter(|restaurant| restaurant.distance_to(location) <= 1.0)
                .map(|restaurant| restaurant.id)
                .collect();
            bot.send_message(
                msg.chat.id,
                "В ближайшие к вам рестораны был отправлен запрос, ожидайте ответа",
            )
            .reply_markup(ReplyMarkup::kb_remove())
            .await?;

            {
                let bot = bot.clone();
                let msg = msg.clone();
                let closest_restaurants_ids = closest_restaurants_ids.clone();
                tokio::spawn(async move {
                    let start_time = Utc::now();
                    let time_to_finish = start_time + Duration::from_secs(120);
                    let mut answered_restaurants = Vec::<u64>::new();
                    loop {
                        let current_time = Utc::now();
                        if current_time < time_to_finish {
                            if answered_restaurants.len() == closest_restaurants_ids.len() {
                                break;
                            }
                            let mut no_answers = 0;
                            for id in &closest_restaurants_ids {
                                if let Some(booking_info) =
                                    restaurants_booking_info.get_async(&id).await
                                {
                                    if *booking_info & (1 << person_number) != 0 {
                                        answered_restaurants.push(*id);
                                    }
                                    if (current_time - start_time).num_seconds() > 30
                                        && *booking_info & (1 << (8 + person_number)) == 0
                                    {
                                        no_answers += 1;
                                    }
                                }
                            }
                            if no_answers == closest_restaurants_ids.len() {
                                break;
                            }
                        } else {
                            break;
                        }
                        task::sleep(Duration::from_secs(1)).await;
                    }
                    if answered_restaurants.len() != 0 {
                        bot.send_message(msg.chat.id, format!("Список ресторанов, где есть места на {person_number} персон: {answered_restaurants:?}")).await?;
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            format!("К сожалению, мест на {person_number} персон нет"),
                        )
                        .await?;
                    }
                    Ok::<(), Error>(())
                });
            }

            let cmd = MestCheckCommand::Check {
                person_number,
                restaurant_ids: closest_restaurants_ids,
            };
            if let Err(err) = sender.send(cmd).await {
                println!("{err:?}")
            };
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.from().unwrap().id, "Отправьте локацию для поиска")
                .await?;
        }
    }

    Ok(())
}
