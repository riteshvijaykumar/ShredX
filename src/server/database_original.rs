use sqlx::{PgPool, Row};
use chrono::Utc;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use        let user = sqlx::query_as::<_, ServerUser>(
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password_hash, created_at, last_login, is_active
            "#
        )
        .bind(&req.id)
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)er::models::*;

pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        let manager = Self { pool };
        manager.create_tables().await?;
        
        Ok(manager)
    }
    
    async fn create_tables(&self) -> Result<(), sqlx::Error> {
        // Create users table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(255) UNIQUE NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                last_login TIMESTAMPTZ,
                is_active BOOLEAN DEFAULT TRUE
            )
        "#)
        .execute(&self.pool)
        .await?;
        
        // Create certificates table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS certificates (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                certificate_data TEXT NOT NULL,
                device_info VARCHAR(500) NOT NULL,
                sanitization_method VARCHAR(100) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                file_hash VARCHAR(255) NOT NULL
            )
        "#)
        .execute(&self.pool)
        .await?;
        
        // Create sanitization logs table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS sanitization_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL REFERENCES users(id),
                certificate_id UUID REFERENCES certificates(id),
                device_path VARCHAR(500) NOT NULL,
                device_type VARCHAR(100) NOT NULL,
                method VARCHAR(100) NOT NULL,
                status VARCHAR(50) NOT NULL,
                duration_seconds BIGINT,
                bytes_processed BIGINT,
                error_message TEXT,
                started_at TIMESTAMPTZ DEFAULT NOW(),
                completed_at TIMESTAMPTZ
            )
        "#)
        .execute(&self.pool)
        .await?;
        
        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_certificates_user_id ON certificates(user_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_user_id ON sanitization_logs(user_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_logs_started_at ON sanitization_logs(started_at)")
            .execute(&self.pool).await?;
        
        Ok(())
    }
    
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<ServerUser, sqlx::Error> {
        let password_hash = Self::hash_password(&req.password);
        let user_id = Uuid::new_v4();
        
        let user = sqlx::query_as!(
            ServerUser,
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password_hash, created_at, last_login, is_active
            "#,
            user_id,
            req.username,
            req.email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn authenticate_user(&self, req: LoginRequest) -> Result<Option<ServerUser>, sqlx::Error> {
        let password_hash = Self::hash_password(&req.password);
        
        let user = sqlx::query_as!(
            ServerUser,
            r#"
            SELECT id, username, email, password_hash, created_at, last_login, is_active
            FROM users 
            WHERE username = $1 AND password_hash = $2 AND is_active = TRUE
            "#,
            req.username,
            password_hash
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if user.is_some() {
            // Update last login
            sqlx::query!(
                "UPDATE users SET last_login = NOW() WHERE username = $1",
                req.username
            )
            .execute(&self.pool)
            .await?;
        }
        
        Ok(user)
    }
    
    pub async fn store_certificate(&self, user_id: Uuid, req: SubmitCertificateRequest) -> Result<Certificate, sqlx::Error> {
        let cert_id = Uuid::new_v4();
        let file_hash = {
            let mut hasher = Sha256::new();
            hasher.update(req.certificate_data.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        
        let certificate = sqlx::query_as!(
            Certificate,
            r#"
            INSERT INTO certificates (id, user_id, certificate_data, device_info, sanitization_method, file_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, certificate_data, device_info, sanitization_method, created_at, file_hash
            "#,
            cert_id,
            user_id,
            req.certificate_data,
            req.device_info,
            req.sanitization_method,
            file_hash
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(certificate)
    }
    
    pub async fn get_user_certificates(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<CertificateResponse, sqlx::Error> {
        let certificates = sqlx::query_as!(
            Certificate,
            r#"
            SELECT id, user_id, certificate_data, device_info, sanitization_method, created_at, file_hash
            FROM certificates 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM certificates WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        
        Ok(CertificateResponse {
            certificates,
            total,
        })
    }
    
    pub async fn log_sanitization(&self, user_id: Uuid, log: SanitizationLog) -> Result<SanitizationLog, sqlx::Error> {
        let log_id = Uuid::new_v4();
        
        let result = sqlx::query_as!(
            SanitizationLog,
            r#"
            INSERT INTO sanitization_logs 
            (id, user_id, certificate_id, device_path, device_type, method, status, 
             duration_seconds, bytes_processed, error_message, started_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, user_id, certificate_id, device_path, device_type, method, status,
                      duration_seconds, bytes_processed, error_message, started_at, completed_at
            "#,
            log_id,
            user_id,
            log.certificate_id,
            log.device_path,
            log.device_type,
            log.method,
            log.status,
            log.duration_seconds,
            log.bytes_processed,
            log.error_message,
            log.started_at,
            log.completed_at
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn get_user_logs(&self, user_id: Uuid, limit: i64, offset: i64) -> Result<SanitizationLogResponse, sqlx::Error> {
        let logs = sqlx::query_as!(
            SanitizationLog,
            r#"
            SELECT id, user_id, certificate_id, device_path, device_type, method, status,
                   duration_seconds, bytes_processed, error_message, started_at, completed_at
            FROM sanitization_logs 
            WHERE user_id = $1
            ORDER BY started_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM sanitization_logs WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        
        Ok(SanitizationLogResponse {
            logs,
            total,
        })
    }
}