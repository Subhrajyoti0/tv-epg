use crate::sqlite::DbPool;

pub struct AuditRepository {
    pool: DbPool,
}

impl AuditRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn log(
        &self,
        event_type: &str,
        entity_type: &str,
        entity_id: Option<&str>,
        message: &str,
        metadata: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let metadata_json = match metadata {
            Some(value) => Some(serde_json::to_string(&value)?),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                event_type,
                entity_type,
                entity_id,
                message,
                metadata_json
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(event_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(message)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
