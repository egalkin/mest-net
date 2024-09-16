#[derive(Clone)]
pub(crate) struct MestCheckCommand {
    pub person_number: u8,
    pub longitude: f64,
    pub latitude: f64,
}

impl MestCheckCommand {
    pub(crate) fn new(person_number: u8, longitude: f64, latitude: f64) -> Self {
        Self {
            person_number,
            longitude,
            latitude,
        }
    }
}
