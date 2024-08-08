use crate::model::commands::MestCheckCommand;
use crate::model::types::Db;
use crate::utils::keyboard::make_request_answer_keyboard;
use anyhow::Result;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub(crate) async fn send_mest_check_notification(
    bot: Bot,
    mut rx: Receiver<MestCheckCommand>,
    restaurants_booking_info: Db<u64, u16>,
    restaurant_managers: Db<u64, UserId>,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            MestCheckCommand::Check {
                person_number,
                restaurant_ids,
            } => {
                let mut set: JoinSet<Result<()>> = JoinSet::new();
                for id in &restaurant_ids {
                    let bot = bot.clone();
                    if let Some(mut booking_info) = restaurants_booking_info.get_async(id).await {
                        if *booking_info & (1 << person_number) != 0 {
                            continue;
                        }
                        if *booking_info & (1 << (8 + person_number)) == 0 {
                            *booking_info |= 1 << (8 + person_number);
                            {
                                let id = *id;
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
