use crate::restaurant::Restaurant;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn deserialize_restaurants<P: AsRef<Path>>(path: P) -> Result<Vec<Restaurant>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let restaurants = serde_json::from_reader(reader)?;
    Ok(restaurants)
}
