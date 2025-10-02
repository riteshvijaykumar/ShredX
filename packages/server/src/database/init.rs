use std::sync::Arc;
use sqlx::PgPool;
use sha2::{Sha256, Digest};
use uuid::Uuid;

pub async fn init_database(pool: Arc<PgPool>) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(255) UNIQUE NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role VARCHAR(50) NOT NULL DEFAULT 'user',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            last_login TIMESTAMP WITH TIME ZONE,
            is_active BOOLEAN DEFAULT true
        )
        "#,
    )
    .execute(pool.as_ref())
    .await?;

    // Create sanitization_jobs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sanitization_jobs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID REFERENCES users(id),
            drive_ids TEXT[] NOT NULL,
            method VARCHAR(50) NOT NULL,
            passes INTEGER NOT NULL,
            status VARCHAR(50) NOT NULL DEFAULT 'pending',
            progress REAL DEFAULT 0.0,
            started_at TIMESTAMP WITH TIME ZONE,
            completed_at TIMESTAMP WITH TIME ZONE,
            error_message TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool.as_ref())
    .await?;

    // Create audit_logs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID REFERENCES users(id),
            action VARCHAR(255) NOT NULL,
            resource_type VARCHAR(100),
            resource_id VARCHAR(255),
            details JSONB,
            ip_address INET,
            user_agent TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool.as_ref())
    .await?;

    // Insert default admin user if not exists
    let admin_password = hash_password("admin123");
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind("admin")
    .bind("admin@hddtool.local")
    .bind(&admin_password)
    .bind("admin")
    .execute(pool.as_ref())
    .await?;

    // Insert default user if not exists
    let user_password = hash_password("user123");
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind(Uuid::new_v4())
    .bind("user")
    .bind("user@hddtool.local")
    .bind(&user_password)
    .bind("user")
    .execute(pool.as_ref())
    .await?;

    tracing::info!("✅ Database initialized successfully");
    Ok(())
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}