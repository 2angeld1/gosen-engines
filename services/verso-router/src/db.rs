use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

#[derive(Clone, Debug)]
pub struct DbPool {
    pub pool: PgPool,
}

impl DbPool {
    pub async fn init() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");

        // Ensure table exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS "MigrationRule" (
                id TEXT PRIMARY KEY,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                source_version TEXT NOT NULL,
                target_version TEXT NOT NULL,
                docs_text TEXT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create MigrationRule table");

        tracing::info!("Connected to PostgreSQL database");

        DbPool { pool }
    }

    pub async fn get_rule(&self, source_lang: &str, target_lang: &str, source_version: &str, target_version: &str) -> Option<String> {
        let rec = sqlx::query(
            r#"
            SELECT docs_text FROM "MigrationRule"
            WHERE source_lang = $1 AND target_lang = $2 AND source_version = $3 AND target_version = $4
            "#
        )
        .bind(source_lang)
        .bind(target_lang)
        .bind(source_version)
        .bind(target_version)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        rec.map(|r| sqlx::Row::get(&r, "docs_text"))
    }

    pub async fn save_rule(&self, source_lang: &str, target_lang: &str, source_version: &str, target_version: &str, docs_text: &str) {
        let id = format!("{}_{}_{}_{}", source_lang, target_lang, source_version, target_version);
        let _ = sqlx::query(
            r#"
            INSERT INTO "MigrationRule" (id, source_lang, target_lang, source_version, target_version, docs_text)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE SET docs_text = $6
            "#
        )
        .bind(id)
        .bind(source_lang)
        .bind(target_lang)
        .bind(source_version)
        .bind(target_version)
        .bind(docs_text)
        .execute(&self.pool)
        .await;
    }
}
