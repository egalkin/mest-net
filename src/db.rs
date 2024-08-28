use crate::entity::manager::{self};
use crate::entity::prelude::{Manager, Restaurant};
use crate::entity::restaurant;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use std::env;

#[derive(Clone)]
pub struct DatabaseHandler {
    pub db: DatabaseConnection,
}

type RestaurantModel = crate::entity::restaurant::Model;
type RestaurantActiveModel = crate::entity::restaurant::ActiveModel;
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

    pub async fn find_restaurants_by_ids(&self, restaurant_ids: Vec<i32>) -> Vec<RestaurantModel> {
        Restaurant::find()
            .filter(restaurant::Column::Id.is_in(restaurant_ids))
            .order_by_desc(restaurant::Column::Score)
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                vec![]
            })
    }

    pub async fn find_restaurant_by_id(&self, id: i32) -> Option<RestaurantModel> {
        Restaurant::find_by_id(id)
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    pub async fn count_restaurants(&self) -> u64 {
        Restaurant::find()
            .count(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                0
            })
    }

    pub async fn find_manager_by_token(&self, token: String) -> Option<ManagerModel> {
        Manager::find()
            .filter(manager::Column::Token.eq(token))
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
            .filter(manager::Column::TgId.eq(id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error accessing the database: {:?}", x);
                None
            })
    }

    pub async fn update_restaurant(
        &self,
        restaurant: RestaurantActiveModel,
    ) -> Result<RestaurantModel, DbErr> {
        restaurant.update(&self.db).await
    }

    pub async fn update_manager(&self, manager: ManagerActiveModel) -> Result<ManagerModel, DbErr> {
        manager.update(&self.db).await
    }
}
