use chrono::NaiveTime;
use lazy_static::lazy_static;

pub const SEARCH_REQUEST_MESSAGE: &str = "Найти места";
pub const BOOKING_EXPIRATION_MINUTES: u64 = 5;
pub const BOOKING_REQUEST_EXPIRATION_MINUTES: u64 = 2;
pub const IN_TIME_ANSWER_BONUS: i32 = 3;
pub const NOT_IN_TIME_ANSWER_PENALTY: i32 = 1;
pub const NO_ANSWER_PENALTY: i32 = 3;
pub const MAX_RESTAURANT_SCORE: i32 = 150;
pub const MIN_RESTAURANT_SCORE: i32 = 0;
pub const SEARCH_RADIUS_IN_METERS: u16 = 1000;
pub const FEEDBACK_FORM_URL: &str = "https://forms.yandex.ru/u/66d1a187068ff0091a9c2979/";

lazy_static! {
    pub static ref DAY_END: NaiveTime = NaiveTime::from_hms_milli_opt(23, 59, 59, 0).unwrap();
    pub static ref MIDNIGHT: NaiveTime = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
}
