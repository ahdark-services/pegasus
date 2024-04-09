use sea_orm_migration::prelude::*;

use pegasus_common::database::entities;

use crate::sea_orm::Schema;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());

        manager
            .create_table(schema.create_table_from_entity(entities::pm_forwarding_bot::Entity))
            .await?;

        manager
            .create_table(schema.create_table_from_entity(entities::pm_forwarding_message::Entity))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(entities::pm_forwarding_bot::Entity)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(entities::pm_forwarding_message::Entity)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
