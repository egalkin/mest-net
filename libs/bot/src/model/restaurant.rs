use serde::{Deserialize, Serialize};
use teloxide::types::Location;
use crate::utils::distance::calculate_distance;

#[derive(Debug, Serialize, Deserialize)]
pub struct Restaurant {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Restaurant {
    pub fn distance_to(&self, location: &Location) -> f64 {
        calculate_distance(self.latitude, self.longitude, location.latitude, location.longitude)
    }
}