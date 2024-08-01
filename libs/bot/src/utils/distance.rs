use crate::utils::constants::EARTH_RADIUS;

pub fn calculate_distance(lat_1: f64, lon_1: f64, lat_2: f64, lon_2: f64) -> f64 {
    let lat_1_rad = lat_1.to_radians();
    let lat_2_rad = lat_2.to_radians();
    let lon_1_rad = lon_1.to_radians();
    let lon_2_rad = lon_2.to_radians();

    let x = (lon_2_rad - lon_1_rad) * ((lat_1_rad + lat_2_rad) / 2f64).cos();
    let y = lat_2_rad - lat_1_rad;

    (x * x + y * y).sqrt() * EARTH_RADIUS
}
