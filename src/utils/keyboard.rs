use crate::utils::constants::SEARCH_REQUEST_MESSAGE;

use lazy_static::lazy_static;
use teloxide::types::{ButtonRequest, KeyboardButton, KeyboardMarkup};

use super::constants::{MAX_SUPPORTED_PERSONS, MIN_SUPPORTED_PERSONS};

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

lazy_static! {
    static ref SUPPORTED_PERSONS_VARIANTS: Vec<String> = {
        (MIN_SUPPORTED_PERSONS..=MAX_SUPPORTED_PERSONS)
            .map(|i| i.to_string())
            .collect()
    };
    static ref SEARCH_VARIANTS: Vec<String> = vec![SEARCH_REQUEST_MESSAGE.to_owned()];
    static ref ROLE_VARIANTS: Vec<String> = vec![
        "Обычный пользователь".to_owned(),
        "Администратор".to_owned()
    ];
    static ref ANSWER_VARIANTS: Vec<String> = vec!["Да".to_owned(), "Нет".to_owned()];
}

pub fn make_number_keyboard() -> KeyboardMarkup {
    make_keyborad_from_string(&SUPPORTED_PERSONS_VARIANTS)
}

pub fn make_search_keyboard() -> KeyboardMarkup {
    make_keyborad_from_string(&SEARCH_VARIANTS)
}

pub fn make_role_keyboard() -> KeyboardMarkup {
    make_keyborad_from_string(&ROLE_VARIANTS)
}

pub fn make_answer_keyboard() -> KeyboardMarkup {
    make_keyborad_from_string(&ANSWER_VARIANTS)
}

fn make_keyborad_from_string(variants: &[String]) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    for versions in variants.chunks(3) {
        let row = versions.iter().map(KeyboardButton::new).collect();
        keyboard.push(row);
    }

    let mut markup = KeyboardMarkup::new(keyboard);
    markup.resize_keyboard = Option::from(true);
    markup
}
