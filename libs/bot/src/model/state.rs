#[derive(Clone, Default)]
pub(crate) enum State {
    #[default]
    Start,
    ReceiveSearchRequest,
    ReceivePersonNumber,
    ReceiveLocation {
        person_number: u8
    },
}
