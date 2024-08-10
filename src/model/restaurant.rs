use crate::utils::distance::calculate_distance;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use teloxide::prelude::UserId;
use teloxide::types::Location;
use time::Time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restaurant {
    pub id: u64,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub token: String,
    pub manager_id: UserId,
    pub schedule: Schedule,
    pub maps_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingTime {
    pub start_time: Time,
    pub end_time: Time,
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

    pub fn set_manager_id(&mut self, manager_id: UserId) {
        self.manager_id = manager_id;
    }
}

impl Display for Restaurant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]({})", self.name, self.maps_url)
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
