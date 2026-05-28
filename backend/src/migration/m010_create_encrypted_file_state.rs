use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        // Manifeste de l'état chiffré par tâche : permet la synchronisation incrémentale
        // (ne re-chiffrer que les fichiers source nouveaux ou modifiés). Les chemins sont en clair,
        // cohérent avec le mode « noms de fichiers en clair ».
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS encrypted_file_state (
                id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                task_id    UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                src_path   TEXT NOT NULL,
                size       BIGINT NOT NULL,
                modtime    TEXT NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE (task_id, src_path)
            )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_encrypted_file_state_task ON encrypted_file_state(task_id)",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS encrypted_file_state")
            .await?;
        Ok(())
    }
}
