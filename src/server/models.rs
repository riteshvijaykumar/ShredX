use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ServerUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Certificate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub certificate_data: String, // JSON serialized certificate
    pub device_info: String,      // Device that was sanitized
    pub sanitization_method: String,
    pub created_at: DateTime<Utc>,
    pub file_hash: String,        // Hash of the certificate for integrity
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SanitizationLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub certificate_id: Option<Uuid>,
    pub device_path: String,
    pub device_type: String,
    pub method: String,
    pub status: String,           // "completed", "failed", "in_progress"
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub bytes_processed: Option<i64>,
    pub verification_passed: Option<bool>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitCertificateRequest {
    pub certificate_data: String,
    pub device_info: String,
    pub sanitization_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateResponse {
    pub certificates: Vec<Certificate>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SanitizationLogResponse {
    pub logs: Vec<SanitizationLog>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCertificateRequest {
    pub user_id: Uuid,
    pub certificate_data: String,
    pub device_info: String,
    pub sanitization_method: String,
    pub file_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SanitizationLogRequest {
    pub user_id: Uuid,
    pub certificate_id: Option<Uuid>,
    pub device_path: String,
    pub device_type: String,
    pub method: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub bytes_processed: Option<i64>,
    pub verification_passed: Option<bool>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: "Success".to_string(),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message,
        }
    }
}