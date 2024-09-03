#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum State {
    #[default]
    Start,
    RoleSelection,
    /// Admin flow
    ReceiveAdminToken,
    ReceiveShareContactAllowance,
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
