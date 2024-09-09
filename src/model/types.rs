use crate::model::state::State;
use anyhow::Result;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::ErasedStorage, prelude::*};

pub(crate) type MyDialogue = Dialogue<State, ErasedStorage<State>>;
pub(crate) type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub(crate) type Db<K, T> = Arc<scc::HashMap<K, T>>;
