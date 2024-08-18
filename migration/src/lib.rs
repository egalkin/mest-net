pub use sea_orm_migration::prelude::*;
mod m20240816_222336_create_restaurant_table;
mod m20240816_222420_create_manager_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240816_222336_create_restaurant_table::Migration),
            Box::new(m20240816_222420_create_manager_table::Migration),
        ]
    }
}
