use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Restaurant::Table)
                    .if_not_exists()
                    .col(pk_auto(Restaurant::Id))
                    .col(string(Restaurant::Name))
                    .col(double(Restaurant::Latitude))
                    .col(double(Restaurant::Longitude))
                    .col(string(Restaurant::MapsUrl))
                    .col(string(Restaurant::AveragePrice))
                    .col(string(Restaurant::Segment))
                    .col(string(Restaurant::Kitchen))
                    .col(json(Restaurant::Schedule))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Restaurant::Table).to_owned())
            .await
    }
}


#[derive(DeriveIden)]
enum Restaurant {
    Table,
    Id,
    Name,
    Latitude,
    Longitude,
    MapsUrl,
    AveragePrice,
    Segment,
    Kitchen,
    Schedule
}
