use crate::utils::constants::{DAY_END, MIDNIGHT};
use crate::utils::distance::calculate_distance;
use chrono::Weekday::{Fri, Sat, Sun};
use chrono::{DateTime, Datelike, NaiveTime, Utc};
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use teloxide::types::Location;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restaurant {
    pub id: i32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub maps_url: String,
    pub average_price: String,
    pub segment: String,
    pub kitchen: String,
    pub schedule: Schedule,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromJsonQueryResult, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum Schedule {
    Regular {
        working_time: WorkingTime,
    },
    WithWeekends {
        weekday_working_time: WorkingTime,
        weekend_working_time: WorkingTime,
    },
}

impl Schedule {
    pub fn match_in(&self, passed_date_time: DateTime<Utc>) -> bool {
        fn match_in(
            passed_date_time: DateTime<Utc>,
            start_time: &NaiveTime,
            end_time: &NaiveTime,
        ) -> bool {
            let time = passed_date_time.time();
            if start_time > end_time {
                return (time >= *start_time && time <= *DAY_END)
                    || (time >= *MIDNIGHT && time <= *end_time);
            } else {
                time >= *start_time && time <= *end_time
            }
        }
        match &self {
            Schedule::Regular { working_time } => match_in(
                passed_date_time,
                &working_time.start_time,
                &working_time.end_time,
            ),
            Schedule::WithWeekends {
                weekday_working_time,
                weekend_working_time,
            } => {
                let week_day = passed_date_time.weekday();
                match week_day {
                    Fri => {
                        let time = passed_date_time.time();
                        if time < weekday_working_time.start_time {
                            match_in(
                                passed_date_time,
                                &weekday_working_time.start_time,
                                &weekday_working_time.end_time,
                            )
                        } else {
                            match_in(
                                passed_date_time,
                                &weekend_working_time.start_time,
                                &weekend_working_time.end_time,
                            )
                        }
                    }
                    Sat => match_in(
                        passed_date_time,
                        &weekend_working_time.start_time,
                        &weekend_working_time.end_time,
                    ),
                    Sun => {
                        let time = passed_date_time.time();
                        if time < weekend_working_time.start_time {
                            match_in(
                                passed_date_time,
                                &weekend_working_time.start_time,
                                &weekend_working_time.end_time,
                            )
                        } else {
                            match_in(
                                passed_date_time,
                                &weekday_working_time.start_time,
                                &weekday_working_time.end_time,
                            )
                        }
                    }
                    _ => match_in(
                        passed_date_time,
                        &weekday_working_time.start_time,
                        &weekday_working_time.end_time,
                    ),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkingTime {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

impl Restaurant {
    pub fn distance_to(&self, location: &Location) -> f64 {
        calculate_distance(
            self.latitude,
            self.longitude,
            location.latitude,
            location.longitude,
        )
    }

    pub fn is_open(&self) -> bool {
        self.schedule.match_in(Utc::now())
    }
}

impl Display for Restaurant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]({}): Кухня: {}; Средний чек: {}",
            self.name, self.maps_url, self.kitchen, self.average_price
        )
    }
}

impl PartialEq for Restaurant {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Restaurant {}

impl Hash for Restaurant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[cfg(test)]
mod tests {

    mod schedule_tests {
        use crate::model::restaurant::Schedule::{Regular, WithWeekends};
        use crate::model::restaurant::WorkingTime;
        use chrono::{DateTime, NaiveTime, TimeZone, Utc};

        #[test]
        fn regular_schedule_one_day_match_in() {
            let start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let end_time = NaiveTime::from_hms_milli_opt(23, 0, 0, 0).unwrap();
            let working_time = WorkingTime {
                start_time,
                end_time,
            };
            let schedule = Regular { working_time };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn regular_schedule_one_day_not_match_in() {
            let start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let end_time = NaiveTime::from_hms_milli_opt(23, 0, 0, 0).unwrap();
            let working_time = WorkingTime {
                start_time,
                end_time,
            };
            let schedule = Regular { working_time };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 1, 23, 30, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn regular_schedule_two_days_match_in() {
            let start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let end_time = NaiveTime::from_hms_milli_opt(3, 0, 0, 0).unwrap();
            let working_time = WorkingTime {
                start_time,
                end_time,
            };
            let schedule = Regular { working_time };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 1, 2, 0, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn regular_schedule_two_days_not_match_in() {
            let start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let end_time = NaiveTime::from_hms_milli_opt(3, 0, 0, 0).unwrap();
            let working_time = WorkingTime {
                start_time,
                end_time,
            };
            let schedule = Regular { working_time };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 1, 4, 0, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_weekday_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 10, 16, 0, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_weekday_not_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 10, 2, 0, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_friday_as_weekday_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 12, 0, 30, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_friday_as_weekday_not_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 12, 1, 30, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_friday_as_weekend_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 12, 16, 30, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_saturday_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 13, 5, 30, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_saturday_not_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 13, 6, 30, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_sunday_as_weekend_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 14, 5, 30, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_sunday_as_weekend_not_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 14, 6, 30, 0).unwrap();

            assert_eq!(false, schedule.match_in(current_date_time))
        }

        #[test]
        fn with_weekends_schedule_sunday_as_weekday_match_in() {
            let weekday_start_time = NaiveTime::from_hms_milli_opt(8, 0, 0, 0).unwrap();
            let weekday_end_time = NaiveTime::from_hms_milli_opt(1, 0, 0, 0).unwrap();
            let weekday_working_time = WorkingTime {
                start_time: weekday_start_time,
                end_time: weekday_end_time,
            };

            let weekend_start_time = NaiveTime::from_hms_milli_opt(12, 0, 0, 0).unwrap();
            let weekend_end_time = NaiveTime::from_hms_milli_opt(6, 0, 0, 0).unwrap();
            let weekend_working_time = WorkingTime {
                start_time: weekend_start_time,
                end_time: weekend_end_time,
            };

            let schedule = WithWeekends {
                weekday_working_time,
                weekend_working_time,
            };

            let current_date_time: DateTime<Utc> =
                Utc.with_ymd_and_hms(2024, 1, 14, 16, 30, 0).unwrap();

            assert_eq!(true, schedule.match_in(current_date_time))
        }
    }
}
