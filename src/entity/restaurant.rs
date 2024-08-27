//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use std::fmt::{Display, Formatter};

use crate::model::restaurant::{Restaurant, Schedule};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "restaurant")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(column_type = "Double")]
    pub latitude: f64,
    #[sea_orm(column_type = "Double")]
    pub longitude: f64,
    pub maps_url: String,
    pub average_price: String,
    pub segment: String,
    pub kitchen: String,
    pub schedule: Schedule,
    #[sea_orm(column_type = "Double")]
    pub score: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::manager::Entity")]
    Manager,
}

impl Related<super::manager::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Manager.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&Model> for Restaurant {
    fn from(value: &Model) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            latitude: value.latitude,
            longitude: value.longitude,
            maps_url: value.maps_url.clone(),
            average_price: value.average_price.clone(),
            segment: value.segment.clone(),
            kitchen: value.kitchen.clone(),
            schedule: value.schedule.clone(),
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<a href=\"{}\">{}</a> — Кухня: {}; Средний чек: {}",
            self.maps_url, self.name, self.kitchen, self.average_price
        )
    }
}
