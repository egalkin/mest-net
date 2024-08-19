//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "manager")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub tg_id: Option<i64>,
    pub token: String,
    pub restaurant_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::restaurant::Entity",
        from = "Column::RestaurantId",
        to = "super::restaurant::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Restaurant,
}

impl Related<super::restaurant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Restaurant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}