use std::sync::Arc;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    utils::command::BotCommands,
};
use teloxide::types::ReplyMarkup;
use crate::model::{
    state::State,
    commands::Command,
    types::*,
    restaurant::Restaurant
};
use crate::utils::keyboard::*;
use crate::utils::constants::SEARCH_REQUEST_MESSAGE;

pub(crate) fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start))
                .branch(case![Command::Reset]).endpoint(reset)
                .branch(dptree::endpoint(invalid_input))
        )
        .branch(case![Command::Reset]).endpoint(reset);
    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveSearchRequest].endpoint(receive_search_request))
        .branch(case![State::ReceivePersonNumber].endpoint(receive_person_number))
        .branch(case![State::ReceiveLocation { person_number }].endpoint(receive_location))
        .branch(dptree::endpoint(invalid_input));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

async fn invalid_input(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Please, send /start.").await?;
    Ok(())
}

/// COMMAND HANDLERS
async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Добро пожаловать в наш бот!")
        .reply_markup(make_search_keyboard())
        .await?;
    dialogue.update(State::ReceiveSearchRequest).await?;
    Ok(())
}

async fn reset(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .reply_markup(ReplyMarkup::kb_remove()).await?;
    dialogue.exit().await?;
    Ok(())
}


/// STATE HANDLERS
async fn receive_search_request(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(SEARCH_REQUEST_MESSAGE) => {
            bot.send_message(msg.chat.id, "Сколько гостей будет?")
                .reply_markup(make_number_keyboard())
                .await?;
            dialogue.update(State::ReceivePersonNumber).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Некорректная команда").await?;
        }
    }

    Ok(())
}

async fn receive_person_number(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(person_number)) => {
            bot.send_message(msg.chat.id, "Отправьте локацию для поиска мест")
                .reply_markup(make_location_keyboard()).await?;
            dialogue.update(State::ReceiveLocation { person_number }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Отправьте число гостей").await?;
        }
    }

    Ok(())
}

async fn receive_location(
    restaurants: Arc<Vec<Restaurant>>,
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
) -> HandlerResult {
    msg.chat.id;
    match msg.location() {
        Some(location) => {
            let closest_restaurants: Vec<&Restaurant> = restaurants.iter()
                .filter(|restaurant| restaurant.distance_to(location) <= 1.0)
                .collect();
            bot.send_message(msg.chat.id, "В ближайшие к вам рестораны был отправлен запрос, ожидайте ответа")
                .reply_markup(ReplyMarkup::kb_remove()).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Отправьте локацию для поиска").await?;
        }
    }

    Ok(())
}
