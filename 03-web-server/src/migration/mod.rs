use sea_orm_migration::prelude::*;

mod m20260421_add_auth;
mod m20260422_100609_add_portfolio;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260421_add_auth::Migration),
            Box::new(m20260422_100609_add_portfolio::Migration),
        ]
    }
}
