use sea_orm_migration::prelude::*;
use std::env;

#[async_std::main]
async fn main() {
    let ref settings = pegasus_common::settings::Settings::read_from_default_file().unwrap();
    let database_url =
        pegasus_common::database::utils::database_url(settings.database.as_ref().unwrap());
    env::set_var("DATABASE_URL", database_url);

    cli::run_cli(pegasus_migration::Migrator).await;
}
