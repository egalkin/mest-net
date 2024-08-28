use crate::background_processing::tasks::wait_for_restaurants_response;
use crate::db::DatabaseHandler;
use crate::model::booking_info::BookingInfo;
use crate::model::commands::BotCommand;
use crate::model::commands::MestCheckCommand;
use crate::model::state::State::Start;
use crate::model::{restaurant::Restaurant, state::State, types::*};
use crate::utils::constants::BOOKING_EXPIRATION_MINUTES;
use crate::utils::constants::IN_TIME_ANSWER_BONUS;
use crate::utils::constants::NOT_IN_TIME_ANSWER_PENALTY;
use crate::utils::constants::SEARCH_REQUEST_MESSAGE;
use crate::utils::keyboard::*;
use chrono::Local;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use std::sync::Arc;
use std::time::Duration;
use teloxide::dispatching::dialogue::ErasedStorage;
use teloxide::types::{ParseMode, ReplyMarkup};
use teloxide::{
    dispatching::{dialogue, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};
use tokio::sync::mpsc::Sender;

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
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

    dialogue::enter::<Update, ErasedStorage<State>, State, _>().branch(message_handler)
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

async fn receive_role_selection(
    restaurants_number: u64,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some("Обычный пользователь") => {
            bot.send_message(
                msg.chat.id,
                format!(include_str!("resources/greetings.txt"), restaurants_number),
            )
            .reply_markup(make_search_keyboard())
            .parse_mode(ParseMode::Html)
            .await?;
            dialogue.update(State::ReceiveSearchRequest).await?;
        }
        Some("Администратор") => {
            bot.send_message(
                msg.chat.id,
                include_str!("resources/greetings_for_admin.txt"),
            )
            .reply_markup(ReplyMarkup::kb_remove())
            .parse_mode(ParseMode::Html)
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
    db_handler: DatabaseHandler,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(token) => match db_handler.find_manager_by_token(token.to_string()).await {
            Some(token_manager) => {
                let token_manager_id = token_manager.id;
                let mut token_manager = token_manager.into_active_model();
                match db_handler
                    .find_manager_by_tg_id(msg.from().unwrap().id.0 as i64)
                    .await
                {
                    Some(tg_id_manager) if token_manager_id != tg_id_manager.id => {
                        bot.send_message(
                            msg.chat.id,
                            "Нельзя быть администратором более чем в одном ресторане",
                        )
                        .await?;
                    }
                    _ => {
                        if token_manager.tg_id.unwrap().is_none() {
                            token_manager.tg_id = Set(Some(msg.from().unwrap().id.0 as i64));
                            db_handler.update_manager(token_manager).await?;
                        }
                        bot.send_message(msg.chat.id, "Ожидайте запросов на бронирование")
                            .await?;
                        dialogue.update(State::WaitingForRequests).await?
                    }
                }
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
    restaurants_booking_info: Db<i32, BookingInfo>,
    db_handler: DatabaseHandler,
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    if let Some(reply_to_message) = msg.reply_to_message() {
        if let Some(text) = reply_to_message.text() {
            if !text.starts_with("У вас есть места на")
                || reply_to_message.from().unwrap().id == msg.from().unwrap().id
            {
                bot.send_message(msg.chat.id, "Выбрано неподходящее сообщение для Reply")
                    .await?;
                return Ok(());
            }
            let tokens = text.split_ascii_whitespace().collect::<Vec<&str>>();
            if let Ok(person_number) = tokens[tokens.len() - 2].parse::<u8>() {
                match msg.text() {
                    Some(ans) if ans == "Да" || ans == "Нет" => {
                        if let Some(manager) = db_handler
                            .find_manager_by_tg_id(msg.from().unwrap().id.0 as i64)
                            .await
                        {
                            if let Some(mut booking_info) = restaurants_booking_info
                                .get_async(&manager.restaurant_id)
                                .await
                            {
                                if ans == "Да" {
                                    booking_info.booking_state |= 1 << person_number;
                                    booking_info.set_booking_expiration_time(
                                        (person_number - 1) as usize,
                                        Local::now()
                                            + Duration::from_secs(BOOKING_EXPIRATION_MINUTES * 60),
                                    );
                                }
                                if let Some(restaurant) = db_handler
                                    .find_restaurant_by_id(manager.restaurant_id)
                                    .await
                                {
                                    let booking_request_expiration_time = booking_info
                                        .get_booking_request_expiration_time(
                                            (person_number - 1) as usize,
                                        );
                                    let current_score_value = restaurant.score;
                                    let mut restaurant = restaurant.into_active_model();
                                    if Local::now() > *booking_request_expiration_time {
                                        restaurant.score =
                                            Set(current_score_value - NOT_IN_TIME_ANSWER_PENALTY);
                                    } else {
                                        restaurant.score =
                                            Set(current_score_value + IN_TIME_ANSWER_BONUS);
                                    }
                                    db_handler.update_restaurant(restaurant).await?;
                                }
                                log::info!(
                                            "{} manager with username = {:?} and user_id = {} {} booking request for {} persons",
                                            booking_info.restaurant_name,
                                            msg.from().unwrap().username,
                                            msg.from().unwrap().id,
                                            if ans == "Да" {"approved"} else {"reject"},
                                            person_number
                                    );
                                booking_info.notifications_state &= !(1 << person_number);
                            }
                        }
                    }
                    _ => {
                        bot.send_message(msg.chat.id, "Ответьте Да или Нет").await?;
                    }
                }
            }
        }
    } else {
        bot.send_message(msg.chat.id, "Отправьте ответ исполоьзуя Reply")
            .await?;
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
    restaurants: Arc<Vec<Arc<Restaurant>>>,
    restaurants_booking_info: Db<i32, BookingInfo>,
    sender: Sender<MestCheckCommand>,
    db_handler: DatabaseHandler,
    bot: Bot,
    dialogue: MyDialogue,
    person_number: u8,
    msg: Message,
) -> HandlerResult {
    match msg.location() {
        Some(location) => {
            let closest_restaurants: Arc<Vec<Arc<Restaurant>>> = Arc::new(
                restaurants
                    .iter()
                    .filter(|restaurant| restaurant.distance_to(location) <= 1.0)
                    .filter(|restaurant| restaurant.is_open())
                    .cloned()
                    .collect(),
            );

            bot.send_message(
                msg.chat.id,
                "В ближайшие к вам рестораны был отправлен запрос, ожидайте ответа",
            )
            .reply_markup(make_search_keyboard())
            .await?;

            {
                let bot = bot.clone();
                let msg = msg.clone();
                let closest_restaurants = closest_restaurants.clone();
                tokio::spawn(async move {
                    wait_for_restaurants_response(
                        bot,
                        msg,
                        db_handler.clone(),
                        closest_restaurants,
                        restaurants_booking_info,
                        person_number,
                    )
                    .await
                });
            }

            let cmd = MestCheckCommand::Check {
                person_number,
                restaurants: closest_restaurants.clone(),
            };
            if let Err(err) = sender.send(cmd).await {
                log::error!("{err}")
            } else {
                log::info!(
                    "User with username = {:?} and user_id = {} send booking request for {} persons at location with latitude = {} and longitude = {}",
                    msg.from().unwrap().username,
                    msg.from().unwrap().id,
                    person_number,
                    location.latitude,
                    location.longitude
                )
            };

            dialogue.update(State::ReceiveSearchRequest).await?;
        }
        None => {
            bot.send_message(msg.from().unwrap().id, "Отправьте локацию для поиска")
                .await?;
        }
    }

    Ok(())
}
