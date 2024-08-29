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
                    .col(string(Restaurant::MapsUrl))
                    .col(string(Restaurant::AveragePrice))
                    .col(string(Restaurant::Segment))
                    .col(string(Restaurant::Kitchen))
                    .col(json(Restaurant::Schedule))
                    .col(integer(Restaurant::Score).default(100))
                    .to_owned(),
            )   
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            "ALTER TABLE restaurant ADD geo_tag geography NOT NULL"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX restaurant_geo_tag_index ON restaurant USING gist(geo_tag)"
        ).await?;

        Ok(())
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
    MapsUrl,
    AveragePrice,
    Segment,
    Kitchen,
    Schedule,
    Score
}
