use chrono::{DateTime, Local};

#[derive(Debug)]
pub(crate) struct BookingInfo {
    pub booking_state: u8,
    pub notifications_state: u8,
    pub booking_request_expiration_times: [DateTime<Local>; 8],
    pub booking_expiration_times: [DateTime<Local>; 8],
    pub restaurant_name: String,
}

impl BookingInfo {
    pub(crate) fn new(restaurant_name: String) -> Self {
        BookingInfo {
            booking_state: 0,
            notifications_state: 0,
            booking_request_expiration_times: [DateTime::default(); 8],
            booking_expiration_times: [DateTime::default(); 8],
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
