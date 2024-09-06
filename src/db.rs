use crate::entity::manager::{self};
use crate::entity::prelude::{Manager, Restaurant};
use crate::entity::restaurant::{self, RestaurantWithManagerInfo};
use crate::utils::constants::SEARCH_RADIUS_IN_METERS;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::{Alias, IntoCondition};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection,
    DbBackend, DbErr, EntityTrait, ExecResult, IntoSimpleExpr, JoinType, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, Statement,
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
    ) -> Vec<RestaurantWithManagerInfo> {
        log::info!("Fetching closest restaurant with longtitude = {}, latitude = {} in radius of {} meters", longitude, latitude, SEARCH_RADIUS_IN_METERS);
        Restaurant::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"select r.*, m.tg_id manager_tg_id, m.share_contact share_manager_contact from restaurant r 
                        inner join manager m on r.id = m.restaurant_id and m.tg_id is not null
                        where ST_DWithin(r.geo_tag, ST_MakePoint($1, $2)::geography, $3) order by score desc, id asc"#,
                [longitude.into(), latitude.into(), SEARCH_RADIUS_IN_METERS.into()],
            ))
            .into_model::<RestaurantWithManagerInfo>()
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

    pub async fn find_restaurants_by_ids(&self, ids: Vec<i32>) -> Vec<RestaurantWithManagerInfo> {
        log::info!("Fetching restaurants by ids");
        Restaurant::find()
            .column_as(
                Expr::col((Alias::new("m"), manager::Column::TgId)).into_simple_expr(),
                "manager_tg_id",
            )
            .column_as(
                Expr::col((Alias::new("m"), manager::Column::ShareContact)).into_simple_expr(),
                "share_manager_contact",
            )
            .join_as(
                JoinType::InnerJoin,
                restaurant::Relation::Manager
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, manager::Column::TgId))
                            .is_not_null()
                            .into_condition()
                    }),
                Alias::new("m"),
            )
            .filter(restaurant::Column::Id.is_in(ids))
            .order_by_desc(restaurant::Column::Score)
            .order_by_asc(restaurant::Column::Id)
            .into_model::<RestaurantWithManagerInfo>()
            .all(&self.db)
            .await
            .unwrap_or_else(|x| {
                log::error!("Error while fetching fetching restaurants by ids: {:?}", x);
                vec![]
            })
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

    pub async fn update_restaurant_score_wiht_raw_sql(
        &self,
        id: i32,
        score: i32,
    ) -> Result<ExecResult, DbErr> {
        log::info!("Set score = {} for restaurant with id = {}", score, id);
        self.db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "Update restaurant set score = $1 where id = $2",
                [score.into(), id.into()],
            ))
            .await
    }

    pub async fn update_manager(&self, manager: ManagerActiveModel) -> Result<ManagerModel, DbErr> {
        manager.update(&self.db).await
    }
}
