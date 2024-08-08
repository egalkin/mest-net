use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;

#[derive(Clone)]
pub struct DatabaseHandler {
    db: DatabaseConnection,
}

impl DatabaseHandler {
    pub async fn new(uri: String) -> Self {
        let mut opt = ConnectOptions::new(uri);
        opt.sqlx_logging(false);

        let db = Database::connect(opt).await.unwrap();

        DatabaseHandler { db }
    }

    pub async fn from_env() -> Self {
        Self::new(env::var("DATABASE_URL").unwrap()).await
    }
}
