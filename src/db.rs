use crate::entity::manager::{self};
use crate::entity::prelude::{Manager, Restaurant};
use crate::utils::constants::SEARCH_RADIUS_IN_METERS;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, DbBackend, DbErr,
    EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, Statement,
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
        log::info!("Fetching all restaurants");
        Restaurant::find()
            .all(&self.db)
            .await
            .unwrap_or_else(|err| {
                log::error!("Error while fetching all restaurants: {:?}", err);
                vec![]
            })
    }

    pub async fn find_restaurant_by_id(&self, id: i32) -> Option<RestaurantModel> {
        log::info!("Fetching restaurant by id = {}", id);
        Restaurant::find_by_id(id)
            .one(&self.db)
            .await
            .unwrap_or_else(|err| {
                log::error!("Error while fetching restaurant by id = {}: {:?}", id, err);
                None
            })
    }

    pub async fn find_closest_restaurants(
        &self,
        longitude: f64,
        latitude: f64,
    ) -> Vec<RestaurantModel> {
        log::info!("Fetching closest restaurant with longtitude = {}, latitude = {} in radius of {} meters", longitude, latitude, SEARCH_RADIUS_IN_METERS);
        Restaurant::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"select r.* from restaurant r 
                        inner join manager m on r.id = m.restaurant_id 
                        where ST_DWithin(r.geo_tag, ST_MakePoint($1, $2)::geography, $3) and m.tg_id is not null order by score desc"#,
                [longitude.into(), latitude.into(), SEARCH_RADIUS_IN_METERS.into()],
            ))
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error while fetching closest restaurant with longtitude = {}, latitude = {} in radius of {} meters: {:?}", longitude, latitude, SEARCH_RADIUS_IN_METERS, x);
                vec![]
            })
            .into_iter()
            .filter(|restaurant| restaurant.is_open())
            .collect()
    }

    pub async fn count_restaurants(&self) -> u64 {
        log::info!("Counting restaurants numnber");
        Restaurant::find()
            .count(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error while counting restaurants number: {:?}", x);
                0
            })
    }

    pub async fn find_manager_for_restaurant(
        &self,
        restaurant: &RestaurantModel,
    ) -> Option<ManagerModel> {
        restaurant
            .find_related(Manager)
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!(
                    "Error while fetching manager for restaurant with id = {}:  {:?}",
                    restaurant.id,
                    x
                );
                None
            })
    }

    pub async fn find_manager_by_token(&self, token: String) -> Option<ManagerModel> {
        log::info!("Fetching manager by token");
        Manager::find()
            .filter(manager::Column::Token.eq(token))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error while fetching manager by token: {:?}", x);
                None
            })
    }

    pub async fn find_manager_by_restaurant_id(&self, restaurant_id: i32) -> Option<ManagerModel> {
        log::info!("Fetching manager with restaurant_id = {}", restaurant_id);
        Manager::find()
            .filter(manager::Column::RestaurantId.eq(restaurant_id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!(
                    "Error while fetching manager with restaurant_id = {}: {:?}",
                    restaurant_id,
                    x
                );
                None
            })
    }

    pub async fn find_manager_by_tg_id(&self, id: i64) -> Option<ManagerModel> {
        log::info!("Fetching manager with tg_id = {}", id);
        Manager::find()
            .filter(manager::Column::TgId.eq(id))
            .one(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error while fetching manager with tg_id = {}: {:?}", id, x);
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
