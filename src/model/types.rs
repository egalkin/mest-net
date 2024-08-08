use crate::model::state::State;
use anyhow::Result;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

pub(crate) type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub(crate) type HandlerResult = Result<()>;
pub(crate) type Db<K, T> = Arc<scc::HashMap<K, T>>;
