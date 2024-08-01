use teloxide::{
    dispatching::{ dialogue::InMemStorage},
    prelude::*,
};
use crate::model::state::State;

pub(crate) type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub(crate) type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;