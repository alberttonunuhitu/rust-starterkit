pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260518_042700_create_auto_update_at_functions::Migration),
            Box::new(m20260517_132307_create_users_table::Migration),
        ]
    }
}

mod m20260517_132307_create_users_table;
mod m20260518_042700_create_auto_update_at_functions;
