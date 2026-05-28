use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Chiffrement asymétrique de la destination. La clé publique (recipient age) est stockée
        // ici ; la clé privée n'est JAMAIS persistée (saisie à la restauration).
        db.execute_unprepared(
            "ALTER TABLE tasks ADD COLUMN IF NOT EXISTS encryption_enabled BOOLEAN NOT NULL DEFAULT FALSE",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE tasks ADD COLUMN IF NOT EXISTS encryption_public_key TEXT",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE tasks DROP COLUMN IF EXISTS encryption_enabled")
            .await?;
        db.execute_unprepared("ALTER TABLE tasks DROP COLUMN IF EXISTS encryption_public_key")
            .await?;
        Ok(())
    }
}
