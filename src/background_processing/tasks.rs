use crate::db::DatabaseHandler;
use crate::entity::restaurant;
use crate::model::booking_info::BookingInfo;
use crate::model::commands::MestCheckCommand;
use crate::model::types::{Db, HandlerResult};
use crate::utils::constants::{BOOKING_REQUEST_EXPIRATION_MINUTES, NO_ANSWER_PENALTY};
use crate::utils::keyboard::make_request_answer_keyboard;
use anyhow::Result;
use async_std::task;
use chrono::Local;
use sea_orm::{IntoActiveModel, Set};
use std::collections::HashSet;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub(crate) async fn send_mest_check_notification(
    bot: Bot,
    mut rx: Receiver<MestCheckCommand>,
    db_handler: DatabaseHandler,
    restaurants_booking_info: Db<i32, BookingInfo>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            MestCheckCommand::Check {
                person_number,
                longitude,
                latitude,
            } => {
                let restaurants = db_handler
                    .find_closest_restaurants(longitude, latitude)
                    .await;
                let mut set: JoinSet<Result<()>> = JoinSet::new();
                for restaurant in restaurants {
                    let restaurant_id = restaurant.id;
                    let bot = bot.clone();
                    if let Some(mut booking_info) =
                        restaurants_booking_info.get_async(&restaurant.id).await
                    {
                        if booking_info.booking_state & (1 << person_number) != 0 {
                            let booking_expiration_time = booking_info
                                .get_booking_expiration_time((person_number - 1) as usize);
                            if Local::now() > *booking_expiration_time {
                                booking_info.booking_state &= !(1 << person_number)
                            } else {
                                continue;
                            }
                        }
                        let booking_request_expiration_time = booking_info
                            .get_booking_request_expiration_time((person_number - 1) as usize);
                        let request_expired =
                            booking_info.notifications_state & (1 << person_number) != 0
                                && Local::now() > *booking_request_expiration_time;
                        if booking_info.notifications_state & (1 << person_number) == 0
                            || request_expired
                        {
                            booking_info.notifications_state |= 1 << person_number;
                            booking_info.set_booking_request_expiration_time(
                                (person_number - 1) as usize,
                                Local::now()
                                    + Duration::from_secs(BOOKING_REQUEST_EXPIRATION_MINUTES * 60),
                            );
                            if request_expired {
                                let current_score = restaurant.score;
                                let mut restaurant = restaurant.into_active_model();
                                restaurant.score = Set(current_score - NO_ANSWER_PENALTY);
                                let _ = db_handler.update_restaurant(restaurant).await;
                            }
                            {
                                let db_handler = db_handler.clone();
                                let restaurants_booking_info = restaurants_booking_info.clone();
                                set.spawn(async move {
                                    match db_handler.find_manager_by_id(restaurant_id).await {
                                        Some(entity) => {
                                            let person_noun_form = resolve_person_noun_form(person_number);
                                            if let Some(tg_id) = entity.tg_id {
                                                bot.send_message(
                                                    UserId(tg_id as u64),
                                                    format!(
                                                        "У вас есть места на {person_number} {person_noun_form}?"
                                                    ),
                                                )
                                                    .reply_markup(make_request_answer_keyboard())
                                                    .await?;
                                            }
                                        }
                                        None => {
                                            if let Some(mut booking_info) =
                                                restaurants_booking_info.get_async(&restaurant_id).await
                                            {
                                                booking_info.notifications_state &=
                                                    !(1 << person_number);
                                            }
                                        }
                                    }
                                    Ok(())
                                });
                            }
                        }
                    }
                }
                while (set.join_next().await).is_some() {}
            }
        }
    }
}

pub(crate) async fn wait_for_restaurants_response(
    bot: Bot,
    msg: Message,
    db_handler: DatabaseHandler,
    longitude: f64,
    latitude: f64,
    restaurants_booking_info: Db<i32, BookingInfo>,
    person_number: u8,
) -> HandlerResult {
    let start_time = Local::now();
    let time_to_finish = start_time + Duration::from_secs(BOOKING_REQUEST_EXPIRATION_MINUTES * 60);
    let closest_restaurants: Vec<restaurant::Model> = db_handler
        .find_closest_restaurants(longitude, latitude)
        .await;
    let mut answered_restaurants: HashSet<&restaurant::Model> =
        HashSet::<&restaurant::Model>::new();
    loop {
        let current_time = Local::now();
        if current_time < time_to_finish {
            answered_restaurants.clear();
            let mut no_answers = 0;
            for restaurant in &*closest_restaurants {
                if let Some(mut booking_info) =
                    restaurants_booking_info.get_async(&restaurant.id).await
                {
                    if booking_info.booking_state & (1 << person_number) != 0 {
                        let booking_expiration_time =
                            booking_info.get_booking_expiration_time((person_number - 1) as usize);
                        if Local::now() > *booking_expiration_time {
                            booking_info.booking_state &= !(1 << person_number)
                        } else {
                            answered_restaurants.insert(restaurant);
                        }
                    }
                    if (current_time - start_time).num_seconds() > 30
                        && booking_info.booking_state & (1 << person_number) == 0
                        && booking_info.notifications_state & (1 << person_number) == 0
                    {
                        no_answers += 1;
                    }
                }
            }
            if (answered_restaurants.len() + no_answers) == closest_restaurants.len() {
                break;
            }
        } else {
            break;
        }
        task::sleep(Duration::from_secs(1)).await;
    }
    let person_noun_form = resolve_person_noun_form(person_number);
    if !answered_restaurants.is_empty() {
        let mut formatted_answer = String::new();
        for restaurant in answered_restaurants {
            formatted_answer.push_str(&format!("<b>•</b> {}\n", restaurant))
        }
        bot.send_message(
            msg.chat.id,
            format!(
                "Список ресторанов, где есть места на {person_number} {person_noun_form}:\n{formatted_answer}"
            ),
        )
        .disable_web_page_preview(true)
        .parse_mode(ParseMode::Html)
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            format!("К сожалению, мест на {person_number} {person_noun_form} нет"),
        )
        .await?;
    }
    Ok(())
}

fn resolve_person_noun_form<'a>(person_number: u8) -> &'a str {
    match person_number {
        1 => "персону",
        2..=4 => "персоны",
        _ => "персон",
    }
}
