use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE notification_channels \
             ADD COLUMN IF NOT EXISTS language TEXT NOT NULL DEFAULT 'fr'",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE notification_channels \
             ADD COLUMN IF NOT EXISTS templates JSONB NOT NULL DEFAULT '{}'::jsonb",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "ALTER TABLE notification_channels DROP COLUMN IF EXISTS templates",
        )
        .await?;
        db.execute_unprepared("ALTER TABLE notification_channels DROP COLUMN IF EXISTS language")
            .await?;
        Ok(())
    }
}
