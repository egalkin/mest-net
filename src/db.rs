use crate::entity::manager::Column;
use crate::entity::prelude::{Manager, Restaurant};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter,
};
use std::env;

#[derive(Clone)]
pub struct DatabaseHandler {
    pub db: DatabaseConnection,
}

type RestaurantModel = crate::entity::restaurant::Model;
type ManagerModel = crate::entity::manager::Model;
type ManagerActiveModel = crate::entity::manager::ActiveModel;

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

    pub async fn find_manager_by_token(&self, token: String) -> Option<ManagerModel> {
        Manager::find()
            .filter(Column::Token.eq(token))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    pub async fn find_manager_by_id(&self, id: i32) -> Option<ManagerModel> {
        Manager::find_by_id(id)
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    pub async fn find_manager_by_tg_id(&self, id: i64) -> Option<ManagerModel> {
        Manager::find()
            .filter(Column::TgId.eq(id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    pub async fn update_manager(&self, manager: ManagerActiveModel) -> Result<ManagerModel, DbErr> {
        manager.update(&self.db).await
    }
}
