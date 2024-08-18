use crate::entity::prelude::Restaurant;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait};
use std::env;

#[derive(Clone)]
pub struct DatabaseHandler {
    pub db: DatabaseConnection,
}

pub type RestaurantModel = crate::entity::restaurant::Model;

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

    pub async fn get_all_restaurants(&self) -> Vec<RestaurantModel> {
        Restaurant::find().all(&self.db).await.unwrap_or_else(|x| {
            log::error!("Error accessing the database: {:?}", x);
            vec![]
        })
    }
}
