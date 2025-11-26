pub use migration_tenant::Migrator;
pub use migration_tenant::MigratorTrait;

pub use migration_tenant::sea_orm::*;

pub mod entities {
    pub use entity_tenant::*;
}
