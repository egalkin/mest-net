use crate::{
    background_processing::tasks::wait_for_restaurants_response,
    db::DatabaseHandler,
    model::{
        booking_info::BookingInfo,
        bot_command::BotCommand,
        mest_check_command::MestCheckCommand,
        state::State::{self, Start},
        types::*,
    },
    utils::{
        constants::{
            BOOKING_EXPIRATION_MINUTES, FEEDBACK_FORM_URL, IN_TIME_ANSWER_BONUS,
            MAX_RESTAURANT_SCORE, MAX_SUPPORTED_PERSONS, MIN_RESTAURANT_SCORE,
            MIN_SUPPORTED_PERSONS, NOT_IN_TIME_ANSWER_PENALTY, SEARCH_REQUEST_MESSAGE,
        },
        keyboard::*,
    },
};
use chrono::Local;
use sea_orm::{ActiveValue::Set, IntoActiveModel};
use std::time::Duration;
use teloxide::{
    dispatching::{dialogue, dialogue::ErasedStorage, UpdateHandler},
    prelude::*,
    types::{ParseMode, ReplyMarkup},
    utils::command::BotCommands,
};
use tokio::sync::{broadcast, mpsc};

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<BotCommand, _>()
        .branch(
            case![Start]
                .branch(case![BotCommand::Help].endpoint(help))
                .branch(case![BotCommand::Start].endpoint(start))
                .branch(case![BotCommand::Reset].endpoint(reset))
                .branch(case![BotCommand::Feedback].endpoint(feedback))
                .branch(dptree::endpoint(invalid_input)),
        )
        .branch(case![BotCommand::Reset].endpoint(reset))
        .branch(case![BotCommand::Feedback].endpoint(feedback));
    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::RoleSelection].endpoint(receive_role_selection))
        // Admin flow
        .branch(case![State::ReceiveAdminToken].endpoint(receive_admin_token))
        .branch(
            case![State::ReceiveShareContactAllowance].endpoint(receive_share_contact_allowance),
        )
        .branch(case![State::WaitingForRequests].endpoint(receive_booking_request))
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

async fn reset(
    db_handler: DatabaseHandler,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    if let Some(manager) = db_handler
        .find_manager_by_tg_id(msg.from().unwrap().id.0 as i64)
        .await
    {
        let mut manager = manager.into_active_model();
        manager.tg_id = Set(None);
        manager.share_contact = Set(false);
        db_handler.update_manager(manager).await?;
    }

    bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
        .reply_markup(ReplyMarkup::kb_remove())
        .await?;

    dialogue.exit().await?;
    Ok(())
}

async fn feedback(bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        format!(
            "Поделиться обратной связью вы можете, заполнив следующую <a href=\"{}\">форму</a>",
            FEEDBACK_FORM_URL
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;
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
                            bot.send_message(
                                msg.chat.id,
                                "Делиться вашим контактом с пользователями для бронирования?",
                            )
                            .reply_markup(make_answer_keyboard())
                            .await?;
                            dialogue.update(State::ReceiveShareContactAllowance).await?
                        } else {
                            bot.send_message(
                                msg.chat.id,
                                include_str!("resources/admin_already_authorized.txt"),
                            )
                            .await?;
                        }
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

async fn receive_share_contact_allowance(
    db_handler: DatabaseHandler,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(ans) if ans == "Да" || ans == "Нет" => {
            if let Some(manager) = db_handler
                .find_manager_by_tg_id(msg.from().unwrap().id.0 as i64)
                .await
            {
                let mut manager = manager.into_active_model();
                manager.share_contact = Set(ans == "Да");
                db_handler.update_manager(manager).await?;
                bot.send_message(msg.chat.id, "Ожидайте запросы на бронирование")
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::WaitingForRequests).await?
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Ответьте Да или Нет").await?;
        }
    }
    Ok(())
}

async fn receive_booking_request(
    restaurants_booking_info: Db<i32, BookingInfo>,
    db_handler: DatabaseHandler,
    sender: broadcast::Sender<(i32, bool, u8)>,
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
                                    let mut score = restaurant.score;
                                    if booking_info.notifications_state & (1 << person_number) != 0
                                    {
                                        if Local::now() > *booking_request_expiration_time {
                                            score = (score - NOT_IN_TIME_ANSWER_PENALTY)
                                                .max(MIN_RESTAURANT_SCORE);
                                        } else {
                                            score = (score + IN_TIME_ANSWER_BONUS)
                                                .min(MAX_RESTAURANT_SCORE);
                                        }
                                    }
                                    if score != restaurant.score {
                                        db_handler
                                            .update_restaurant_score_wiht_raw_sql(
                                                restaurant.id,
                                                score,
                                            )
                                            .await?;
                                    }
                                }
                                log::info!(
                                    "{} manager with username = {:?} and user_id = {} {} booking \
                                     request for {} persons",
                                    booking_info.restaurant_name,
                                    msg.from().unwrap().username,
                                    msg.from().unwrap().id,
                                    if ans == "Да" { "approved" } else { "reject" },
                                    person_number
                                );
                                booking_info.notifications_state &= !(1 << person_number);
                                if let Err(err) =
                                    sender.send((manager.restaurant_id, ans == "Да", person_number))
                                {
                                    log::error!("{err}");
                                }
                                bot.send_message(msg.chat.id, "Спасибо за ваш ответ")
                                    .await?;
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
        Some(Ok(person_number))
            if (MIN_SUPPORTED_PERSONS..=MAX_SUPPORTED_PERSONS).contains(&person_number) =>
        {
            bot.send_message(msg.chat.id, "Отправьте локацию для поиска мест")
                .reply_markup(make_location_keyboard())
                .await?;
            dialogue
                .update(State::ReceiveLocation { person_number })
                .await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "Отправьте число от {} до {}",
                    MIN_SUPPORTED_PERSONS, MAX_SUPPORTED_PERSONS
                ),
            )
            .await?;
        }
    }

    Ok(())
}

async fn receive_location(
    restaurants_booking_info: Db<i32, BookingInfo>,
    command_sender: mpsc::Sender<MestCheckCommand>,
    answer_sender: broadcast::Sender<(i32, bool, u8)>,
    db_handler: DatabaseHandler,
    bot: Bot,
    dialogue: MyDialogue,
    person_number: u8,
    msg: Message,
) -> HandlerResult {
    match msg.location() {
        Some(location) => {
            bot.send_message(
                msg.chat.id,
                "В ближайшие к вам рестораны был отправлен запрос, ожидайте ответа",
            )
            .reply_markup(make_search_keyboard())
            .await?;

            let mest_check_command =
                MestCheckCommand::new(person_number, location.longitude, location.latitude);
            {
                let bot = bot.clone();
                let msg = msg.clone();
                let mest_check_command = mest_check_command.clone();
                tokio::spawn(async move {
                    wait_for_restaurants_response(
                        bot,
                        msg.chat.id,
                        answer_sender.subscribe(),
                        db_handler.clone(),
                        mest_check_command,
                        restaurants_booking_info,
                    )
                    .await
                });
            }

            if let Err(err) = command_sender.send(mest_check_command.clone()).await {
                log::error!("{err}")
            } else {
                log::info!(
                    "User with username = {:?} and user_id = {} send booking request for {} \
                     persons at location with latitude = {} and longitude = {}",
                    msg.from().unwrap().username,
                    msg.from().unwrap().id,
                    person_number,
                    mest_check_command.latitude,
                    mest_check_command.longitude
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
