use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Restaurant::Table)
                    .add_column(
                        ColumnDef::new(Restaurant::Score)
                            .double()
                            .not_null()
                            .default(100.0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Restaurant::Table)
                    .drop_column(Restaurant::Score)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Restaurant {
    Table,
    Score,
}
