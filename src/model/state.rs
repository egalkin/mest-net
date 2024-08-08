#[derive(Clone, Default)]
pub(crate) enum State {
    #[default]
    Start,
    RoleSelection,
    /// Admin flow
    ReceiveAdminToken,
    WaitingForRequests,
    RequestAnswered {
        person_number: u8,
    },
    /// User flow
    ReceiveSearchRequest,
    ReceivePersonNumber,
    ReceiveLocation {
        person_number: u8,
    },
}
