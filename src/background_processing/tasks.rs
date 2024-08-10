use crate::model::booking_info::BookingInfo;
use crate::model::commands::MestCheckCommand;
use crate::model::restaurant::Restaurant;
use crate::model::types::{Db, HandlerResult};
use crate::utils::keyboard::make_request_answer_keyboard;
use anyhow::Result;
use async_std::task;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub(crate) async fn send_mest_check_notification(
    bot: Bot,
    mut rx: Receiver<MestCheckCommand>,
    restaurants_booking_info: Db<u64, BookingInfo>,
    restaurant_managers: Db<u64, UserId>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            MestCheckCommand::Check {
                person_number,
                restaurants,
            } => {
                let mut set: JoinSet<Result<()>> = JoinSet::new();
                for restaurant in &*restaurants {
                    let bot = bot.clone();
                    if let Some(mut booking_info) =
                        restaurants_booking_info.get_async(&restaurant.id).await
                    {
                        if booking_info.booking_state & (1 << person_number) != 0 {
                            let booking_expiration_time = booking_info
                                .get_booking_expiration_time((person_number - 1) as usize);
                            if Utc::now() > *booking_expiration_time {
                                booking_info.booking_state &= !(1 << person_number)
                            } else {
                                continue;
                            }
                        }
                        let booking_request_expiration_time = booking_info
                            .get_booking_request_expiration_time((person_number - 1) as usize);
                        if booking_info.notifications_state & (1 << person_number) == 0
                            || Utc::now() > *booking_request_expiration_time
                        {
                            booking_info.notifications_state |= 1 << person_number;
                            booking_info.set_booking_request_expiration_time(
                                (person_number - 1) as usize,
                                Utc::now() + Duration::from_secs(30),
                            );
                            {
                                let id = restaurant.id;
                                let restaurant_managers = restaurant_managers.clone();
                                set.spawn(async move {
                                    match restaurant_managers.get_async(&id).await {
                                        Some(entry) => {
                                            bot.send_message(
                                                *entry.get(),
                                                format!(
                                                    "У вас есть места на {person_number} персон?"
                                                ),
                                            )
                                            .reply_markup(make_request_answer_keyboard())
                                            .await?;
                                        }
                                        _ => {}
                                    }
                                    Ok(())
                                });
                            }
                        }
                    }
                }
                while let Some(_) = set.join_next().await {}
            }
        }
    }
}

pub(crate) async fn wait_for_restaurants_response(
    bot: Bot,
    msg: Message,
    closest_restaurants: Arc<Vec<Arc<Restaurant>>>,
    restaurants_booking_info: Db<u64, BookingInfo>,
    person_number: u8,
) -> HandlerResult {
    let start_time = Utc::now();
    let time_to_finish = start_time + Duration::from_secs(120);
    let mut answered_restaurants = Vec::<&Restaurant>::new();
    loop {
        let current_time = Utc::now();
        if current_time < time_to_finish {
            if answered_restaurants.len() == closest_restaurants.len() {
                break;
            }
            let mut no_answers = 0;
            for restaurant in &*closest_restaurants {
                if let Some(mut booking_info) =
                    restaurants_booking_info.get_async(&restaurant.id).await
                {
                    if booking_info.booking_state & (1 << person_number) != 0 {
                        let booking_expiration_time =
                            booking_info.get_booking_expiration_time((person_number - 1) as usize);
                        if Utc::now() > *booking_expiration_time {
                            booking_info.booking_state &= !(1 << person_number)
                        } else {
                            answered_restaurants.push(restaurant);
                        }
                    }
                    if (current_time - start_time).num_seconds() > 30
                        && booking_info.notifications_state & (1 << person_number) == 0
                    {
                        no_answers += 1;
                    }
                }
            }
            if no_answers == closest_restaurants.len() {
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
    Ok(())
}
