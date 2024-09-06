use crate::db::DatabaseHandler;
use crate::entity::restaurant;
use crate::model::booking_info::BookingInfo;
use crate::model::commands::MestCheckCommand;
use crate::model::types::{Db, HandlerResult};
use crate::utils::constants::{
    BOOKING_REQUEST_EXPIRATION_MINUTES, MIN_RESTAURANT_SCORE, NO_ANSWER_PENALTY,
};
use crate::utils::keyboard::make_answer_keyboard;
use anyhow::Result;
use async_std::task;
use chrono::Local;
use scc::hash_map::OccupiedEntry;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

type Restaurant = restaurant::RestaurantWithManagerInfo;

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
                let restaurants: Vec<Restaurant> = db_handler
                    .find_closest_restaurants(longitude, latitude)
                    .await;
                let mut set: JoinSet<Result<()>> = JoinSet::new();
                for restaurant in restaurants {
                    let tg_id = restaurant.manager_tg_id;
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

                        process_request_expirations(
                            db_handler.clone(),
                            &mut booking_info,
                            restaurant,
                        )
                        .await;

                        if booking_info.notifications_state & (1 << person_number) == 0 {
                            booking_info.notifications_state |= 1 << person_number;
                            booking_info.set_booking_request_expiration_time(
                                (person_number - 1) as usize,
                                Local::now()
                                    + Duration::from_secs(BOOKING_REQUEST_EXPIRATION_MINUTES * 60),
                            );
                            set.spawn(async move {
                                let person_noun_form = resolve_person_noun_form(person_number);
                                bot.send_message(
                                    UserId(tg_id as u64),
                                    format!(
                                        "У вас есть места на {person_number} {person_noun_form}?"
                                    ),
                                )
                                .reply_markup(make_answer_keyboard())
                                .await?;
                                Ok(())
                            });
                        }
                    }
                }
                while (set.join_next().await).is_some() {}
            }
        }
    }
}

async fn process_request_expirations(
    db_handler: DatabaseHandler,
    booking_info: &mut OccupiedEntry<'_, i32, BookingInfo>,
    restaurant: Restaurant,
) {
    let current_time = &Local::now();
    let mut total_penalty: i32 = 0;
    for person_number in 1..booking_info.booking_request_expiration_times.len() + 1 {
        let booking_request_expiration_time =
            booking_info.get_booking_request_expiration_time(person_number - 1);
        let request_expired = booking_info.notifications_state & (1 << person_number) != 0
            && *current_time > *booking_request_expiration_time;
        if request_expired {
            total_penalty += NO_ANSWER_PENALTY;
            booking_info.notifications_state &= !(1 << person_number);
        }
    }
    if total_penalty != 0 {
        let score = (restaurant.score - total_penalty).max(MIN_RESTAURANT_SCORE);
        if score != restaurant.score {
            let _ = db_handler
                .update_restaurant_score_wiht_raw_sql(restaurant.id, score)
                .await;
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
    let closest_restaurants: Vec<Restaurant> = db_handler
        .find_closest_restaurants(longitude, latitude)
        .await;
    let mut answered_restaurants_ids: Vec<i32> = Vec::with_capacity(closest_restaurants.len());
    loop {
        let current_time = Local::now();
        if current_time < time_to_finish {
            answered_restaurants_ids.clear();
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
                            answered_restaurants_ids.push(restaurant.id);
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
            if (answered_restaurants_ids.len() + no_answers) == closest_restaurants.len() {
                break;
            }
        } else {
            break;
        }
        task::sleep(Duration::from_secs(1)).await;
    }
    let person_noun_form = resolve_person_noun_form(person_number);
    if !answered_restaurants_ids.is_empty() {
        let answered_restaurants = db_handler
            .find_restaurants_by_ids(answered_restaurants_ids)
            .await;
        let mut formatted_answer = String::new();
        for restaurant in answered_restaurants {
            formatted_answer.push_str(&format!("<b>•</b> {}\n", restaurant));
            if restaurant.share_manager_contact {
                formatted_answer.push_str(&format!(
                    "          <a href=\"tg://user?id={}\">Предупредить о визите</a>\n",
                    restaurant.manager_tg_id
                ));
            } else {
                formatted_answer
                    .push_str(&format!("          Телефон: {}\n", restaurant.phone_number))
            }
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
