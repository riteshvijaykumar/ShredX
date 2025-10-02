use sqlx::PgPool;
use crate::server::models::*;
use sha2::{Sha256, Digest};

pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub async fn create_user(&self, req: CreateUserRequest) -> Result<ServerUser, sqlx::Error> {
        let password_hash = Self::hash_password(&req.password);
        let user_id = uuid::Uuid::new_v4();
        
        let user = sqlx::query_as::<_, ServerUser>(
            r#"
            INSERT INTO users (id, username, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, email, password_hash, created_at, last_login, is_active
            "#
        )
        .bind(&user_id)
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(user)
    }
    
    pub async fn authenticate_user(&self, req: LoginRequest) -> Result<Option<ServerUser>, sqlx::Error> {
        let password_hash = Self::hash_password(&req.password);
        
        let user = sqlx::query_as::<_, ServerUser>(
            r#"
            SELECT id, username, email, password_hash, created_at, last_login, is_active
            FROM users 
            WHERE username = $1 AND password_hash = $2 AND is_active = TRUE
            "#
        )
        .bind(&req.username)
        .bind(&password_hash)
        .fetch_optional(&self.pool)
        .await?;
        
        if user.is_some() {
            // Update last login
            sqlx::query("UPDATE users SET last_login = NOW() WHERE username = $1")
                .bind(&req.username)
                .execute(&self.pool)
                .await?;
        }
        
        Ok(user)
    }
    
    pub async fn store_certificate(&self, req: StoreCertificateRequest) -> Result<Certificate, sqlx::Error> {
        let certificate_id = uuid::Uuid::new_v4();
        
        let certificate = sqlx::query_as::<_, Certificate>(
            r#"
            INSERT INTO certificates (id, user_id, certificate_data, device_info, sanitization_method, file_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, certificate_data, device_info, sanitization_method, created_at, file_hash
            "#
        )
        .bind(&certificate_id)
        .bind(&req.user_id)
        .bind(&req.certificate_data)
        .bind(&req.device_info)
        .bind(&req.sanitization_method)
        .bind(&req.file_hash)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(certificate)
    }
    
    pub async fn get_user_certificates(&self, user_id: uuid::Uuid, limit: i64, offset: i64) -> Result<PaginatedResponse<Certificate>, sqlx::Error> {
        let certificates = sqlx::query_as::<_, Certificate>(
            r#"
            SELECT id, user_id, certificate_data, device_info, sanitization_method, created_at, file_hash
            FROM certificates 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM certificates WHERE user_id = $1"
        )
        .bind(&user_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(PaginatedResponse {
            data: certificates,
            total: total as u64,
            page: (offset / limit + 1) as u64,
            per_page: limit as u64,
        })
    }
    
    pub async fn log_sanitization(&self, log: SanitizationLogRequest) -> Result<SanitizationLog, sqlx::Error> {
        let log_id = uuid::Uuid::new_v4();
        
        let result = sqlx::query_as::<_, SanitizationLog>(
            r#"
            INSERT INTO sanitization_logs
            (id, user_id, certificate_id, device_path, device_type, method, status, 
             started_at, completed_at, bytes_processed, verification_passed, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, user_id, certificate_id, device_path, device_type, method, status,
                     started_at, completed_at, bytes_processed, verification_passed, error_message, created_at
            "#
        )
        .bind(&log_id)
        .bind(&log.user_id)
        .bind(&log.certificate_id)
        .bind(&log.device_path)
        .bind(&log.device_type)
        .bind(&log.method)
        .bind(&log.status)
        .bind(&log.started_at)
        .bind(&log.completed_at)
        .bind(&log.bytes_processed)
        .bind(&log.verification_passed)
        .bind(&log.error_message)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result)
    }
    
    pub async fn get_sanitization_logs(&self, user_id: uuid::Uuid, limit: i64, offset: i64) -> Result<PaginatedResponse<SanitizationLog>, sqlx::Error> {
        let logs = sqlx::query_as::<_, SanitizationLog>(
            r#"
            SELECT id, user_id, certificate_id, device_path, device_type, method, status,
                   started_at, completed_at, bytes_processed, verification_passed, error_message, created_at
            FROM sanitization_logs 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sanitization_logs WHERE user_id = $1"
        )
        .bind(&user_id)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(PaginatedResponse {
            data: logs,
            total: total as u64,
            page: (offset / limit + 1) as u64,
            per_page: limit as u64,
        })
    }
    
    pub async fn get_certificate_by_id(&self, cert_id: uuid::Uuid, user_id: uuid::Uuid) -> Result<Option<Certificate>, sqlx::Error> {
        let certificate = sqlx::query_as::<_, Certificate>(
            r#"
            SELECT id, user_id, device_info, sanitization_method, start_time, end_time, 
                   passes_completed, verification_status, certificate_data, created_at
            FROM certificates 
            WHERE id = $1 AND user_id = $2
            "#
        )
        .bind(&cert_id)
        .bind(&user_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(certificate)
    }
}