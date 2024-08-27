use chrono::NaiveTime;
use lazy_static::lazy_static;

pub const SEARCH_REQUEST_MESSAGE: &str = "Найти места";
pub const EARTH_RADIUS: f64 = 6371.0;

lazy_static! {
    pub static ref DAY_END: NaiveTime = NaiveTime::from_hms_milli_opt(23, 59, 59, 0).unwrap();
    pub static ref MIDNIGHT: NaiveTime = NaiveTime::from_hms_milli_opt(0, 0, 0, 0).unwrap();
}
