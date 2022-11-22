pub use sea_orm_migration::prelude::*;

mod m20220803_000818_create_user;
mod m20220803_000830_create_key;
mod m20220803_000835_create_registration;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220803_000818_create_user::Migration),
            Box::new(m20220803_000830_create_key::Migration),
            Box::new(m20220803_000835_create_registration::Migration),
        ]
    }
}
