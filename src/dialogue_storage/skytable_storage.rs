use bb8::RunError;
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};
use skytable::{error::Error, pool, pool::ConnectionMgrTcp, query, Config};
use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    sync::Arc,
};
use teloxide::{
    dispatching::dialogue::{Serializer, Storage},
    prelude::ChatId,
};
use thiserror::Error;

type SkytablePool = bb8::Pool<ConnectionMgrTcp>;

const ROW_ALREADY_EXISTS_CODE: u16 = 108;
const ROW_NOT_FOUND_CODE: u16 = 111;

#[derive(Debug, Error)]
pub enum SkytableStorageError<SE>
where
    SE: Debug + Display,
{
    #[error("dialogue serialization error: {0}")]
    SerdeError(SE),

    #[error("run error: {0}")]
    RunError(#[from] RunError<Error>),

    #[error("skytable error: {0}")]
    SkytableError(#[from] Error),

    /// Returned from [`SkytableStorage::remove_dialogue`].
    #[error("row not found")]
    DialogueNotFound,
}

pub struct SkytableStorage<S> {
    pool: SkytablePool,
    serializer: S,
}

impl<S> SkytableStorage<S> {
    pub async fn open(
        skytable_host: &str,
        skytable_port: u16,
        skytable_user: &str,
        skytable_password: &str,
        max_connections: u32,
        serializer: S,
    ) -> Result<Arc<Self>, SkytableStorageError<Infallible>> {
        let config = Config::new(
            skytable_host,
            skytable_port,
            skytable_user,
            skytable_password,
        );
        let mut conn = config.connect()?;
        conn.query_parse::<bool>(&query!("create space if not exists mest_net"))?;
        conn.query_parse::<bool>(&query!(
            "create model if not exists mest_net.dialogues(chat_id: uint64, dialogue: binary)"
        ))?;
        let pool = pool::get_async(max_connections, config).await.unwrap();
        Ok(Arc::new(Self { pool, serializer }))
    }

    fn log_unexpected_error(chat_id: i64, err: &Error) {
        log::error!(
            "Unexpected error occurs during fetching dialogue with chat id = {}",
            chat_id
        );
        log::error!("Error description: {:?}", err);
    }
}

impl<S, D> Storage<D> for SkytableStorage<S>
where
    S: Send + Sync + Serializer<D> + 'static,
    D: Send + Serialize + DeserializeOwned + 'static,
    <S as Serializer<D>>::Error: Debug + Display,
{
    type Error = SkytableStorageError<<S as Serializer<D>>::Error>;

    fn remove_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
    ) -> BoxFuture<'static, Result<(), Self::Error>> {
        Box::pin(async move {
            let mut conn = self.pool.get().await.unwrap();

            let delete_result = conn
                .query_parse::<()>(&query!(
                    "delete from mest_net.dialogues where chat_id = ?",
                    chat_id as u64
                ))
                .await;

            match delete_result {
                Ok(_) => {
                    log::info!("Dialogue with chat id = {} successfully deleted", chat_id);
                    Ok(())
                }
                Err(skytable_err) => match skytable_err {
                    Error::ServerError(ROW_NOT_FOUND_CODE) => {
                        log::info!("Dialogue with chat id = {} not found", chat_id);
                        Err(SkytableStorageError::DialogueNotFound)
                    }
                    err => {
                        SkytableStorage::<S>::log_unexpected_error(chat_id, &err);
                        Err(SkytableStorageError::SkytableError(err))
                    }
                },
            }
        })
    }

    fn update_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
        dialogue: D,
    ) -> BoxFuture<'static, Result<(), Self::Error>> {
        Box::pin(async move {
            let d = self
                .serializer
                .serialize(&dialogue)
                .map_err(SkytableStorageError::SerdeError)?;
            let mut conn = self.pool.get().await.unwrap();

            let insert_result = conn
                .query_parse::<()>(&query!(
                    "insert into mest_net.dialogues(?, ?)",
                    chat_id as u64,
                    &d
                ))
                .await;

            match insert_result {
                Ok(_) => {
                    log::info!("Dialogue with chat id = {} successfully inserted", chat_id);
                    Ok(())
                }
                Err(skytable_err) => match skytable_err {
                    Error::ServerError(ROW_ALREADY_EXISTS_CODE) => {
                        let update_result = conn
                            .query_parse::<()>(&query!(
                                "update mest_net.dialogues set dialogue = ? where chat_id = ?",
                                &d,
                                chat_id as u64
                            ))
                            .await;

                        match update_result {
                            Ok(_) => {
                                log::info!(
                                    "Dialogue with chat id = {} successfully updated",
                                    chat_id
                                );
                                Ok(())
                            }
                            Err(err) => {
                                SkytableStorage::<S>::log_unexpected_error(chat_id, &err);
                                Err(SkytableStorageError::SkytableError(skytable_err))
                            }
                        }
                    }
                    err => {
                        SkytableStorage::<S>::log_unexpected_error(chat_id, &err);
                        Err(SkytableStorageError::SkytableError(err))
                    }
                },
            }
        })
    }

    fn get_dialogue(
        self: Arc<Self>,
        ChatId(chat_id): ChatId,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            let mut conn = self.pool.get().await.unwrap();

            let dialogue: Option<Vec<u8>> = match conn
                .query_parse::<(Vec<u8>,)>(&query!(
                    "select dialogue from mest_net.dialogues where chat_id = ?",
                    chat_id as u64
                ))
                .await
            {
                Ok(val) => {
                    log::info!("Dialogue with chat id = {} successfully fetched", chat_id);
                    Some(val.0)
                }
                Err(skytable_err) => match skytable_err {
                    Error::ServerError(ROW_NOT_FOUND_CODE) => {
                        log::info!("Dialogue with chat id = {} not found", chat_id);
                        None
                    }
                    err => {
                        SkytableStorage::<S>::log_unexpected_error(chat_id, &err);
                        None
                    }
                },
            };

            dialogue
                .map(|d| {
                    self.serializer
                        .deserialize(&d)
                        .map_err(SkytableStorageError::SerdeError)
                })
                .transpose()
        })
    }
}
