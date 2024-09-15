use chrono::{DateTime, Local};

use crate::utils::constants::MAX_SUPPORTED_PERSONS;

#[derive(Debug)]
pub(crate) struct BookingInfo {
    pub booking_state: u8,
    pub notifications_state: u8,
    pub booking_request_expiration_times: [DateTime<Local>; MAX_SUPPORTED_PERSONS as usize],
    pub booking_expiration_times: [DateTime<Local>; MAX_SUPPORTED_PERSONS as usize],
    pub restaurant_name: String,
}

impl BookingInfo {
    pub(crate) fn new(restaurant_name: String) -> Self {
        BookingInfo {
            booking_state: 0,
            notifications_state: 0,
            booking_request_expiration_times: [DateTime::default(); MAX_SUPPORTED_PERSONS as usize],
            booking_expiration_times: [DateTime::default(); MAX_SUPPORTED_PERSONS as usize],
            restaurant_name,
        }
    }

    pub(crate) fn get_booking_request_expiration_time(&self, index: usize) -> &DateTime<Local> {
        &self.booking_request_expiration_times[index]
    }

    pub(crate) fn set_booking_request_expiration_time(
        &mut self,
        index: usize,
        time_to_set: DateTime<Local>,
    ) {
        self.booking_request_expiration_times[index] = time_to_set
    }

    pub(crate) fn get_booking_expiration_time(&self, index: usize) -> &DateTime<Local> {
        &self.booking_expiration_times[index]
    }

    pub(crate) fn set_booking_expiration_time(
        &mut self,
        index: usize,
        time_to_set: DateTime<Local>,
    ) {
        self.booking_expiration_times[index] = time_to_set
    }
}
