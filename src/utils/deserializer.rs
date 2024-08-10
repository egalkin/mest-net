use crate::restaurant::Restaurant;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

pub fn deserialize_restaurants<P: AsRef<Path>>(path: P) -> Result<Vec<Arc<Restaurant>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let restaurants: Vec<Restaurant> = serde_json::from_reader(reader)?;
    let restaurants = restaurants
        .iter()
        .map(|restaurant| Arc::new(restaurant.clone()))
        .collect();
    Ok(restaurants)
}
