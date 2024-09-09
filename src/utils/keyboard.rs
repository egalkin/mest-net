use crate::utils::constants::SEARCH_REQUEST_MESSAGE;

use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup};

pub fn make_location_keyboard() -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];
    let mut location_button = KeyboardButton::new("Отправить текущую локацию");
    location_button.request = Some(ButtonRequest::Location);
    let row = vec![location_button];
    keyboard.push(row);
    let mut markup = KeyboardMarkup::new(keyboard);
    markup.resize_keyboard = Option::from(true);
    markup
}

pub fn make_number_keyboard() -> KeyboardMarkup {
    make_keyboard(vec!["1", "2", "3", "4", "5", "6"])
}

pub fn make_search_keyboard() -> KeyboardMarkup {
    make_keyboard(vec![SEARCH_REQUEST_MESSAGE])
}

pub fn make_role_keyboard() -> KeyboardMarkup {
    make_keyboard(vec!["Обычный пользователь", "Администратор"])
}

pub fn make_answer_keyboard() -> KeyboardMarkup {
    make_keyboard(vec!["Да", "Нет"])
}

fn make_keyboard(variants: Vec<&str>) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    for versions in variants.chunks(3) {
        let row = versions
            .iter()
            .map(|&version| KeyboardButton::new(version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    let mut markup = KeyboardMarkup::new(keyboard);
    markup.resize_keyboard = Option::from(true);
    markup
}
