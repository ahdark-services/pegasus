use sea_orm::{Database, DatabaseConnection};

use crate::settings;

pub mod entities;
pub mod utils;

pub async fn init_conn(
    database: &settings::Database,
) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(utils::database_url(database)).await
}
