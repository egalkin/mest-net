use chrono::NaiveTime;
use lazy_static::lazy_static;

pub const SEARCH_REQUEST_MESSAGE: &str = "Найти места";
pub const BOOKING_EXPIRATION_MINUTES: u64 = 5;
pub const BOOKING_REQUEST_EXPIRATION_MINUTES: u64 = 2;
pub const IN_TIME_ANSWER_BONUS: f64 = 1.0;
pub const NOT_IN_TIME_ANSWER_PENALTY: f64 = 0.5;
pub const NO_ANSWER_PENALTY: f64 = 1.0;

lazy_static! {
    pub static ref DAY_END: NaiveTime = NaiveTime::from_hms_milli_opt(23, 59, 59, 0).unwrap();
    pub static ref MIDNIGHT: NaiveTime = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
}
