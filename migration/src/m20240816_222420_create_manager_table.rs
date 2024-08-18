use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Manager::Table)
                    .if_not_exists()
                    .col(pk_auto(Manager::Id))
                    .col(big_integer_null(Manager::TgId).unique_key())
                    .col(string(Manager::Token))
                    .col(integer(Manager::RestaurantId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-manager-restaurant_id")
                            .from(Manager::Table, Manager::RestaurantId)
                            .to(Restaurant::Table, Restaurant::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(Manager::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Manager {
    Table,
    Id,
    TgId,
    Token,
    RestaurantId
}

#[derive(DeriveIden)]
enum Restaurant {
    Table,
    Id
}
