use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub is_authenticated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

#[derive(Clone)]
pub struct ServerClient {
    server_url: String,
    client: reqwest::Client,
    current_session: Option<UserSession>,
}

impl ServerClient {
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            client: reqwest::Client::new(),
            current_session: None,
        }
    }

    pub async fn create_account(&self, request: CreateAccountRequest) -> Result<ApiResponse<UserSession>, Box<dyn std::error::Error>> {
        if request.password != request.confirm_password {
            return Ok(ApiResponse {
                success: false,
                data: None,
                message: "Passwords do not match".to_string(),
            });
        }

        let url = format!("{}/api/auth/register", self.server_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let result: ApiResponse<UserSession> = response.json().await?;
        Ok(result)
    }

    pub async fn login(&mut self, request: LoginRequest) -> Result<ApiResponse<UserSession>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/auth/login", self.server_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let result: ApiResponse<UserSession> = response.json().await?;
        
        if result.success {
            if let Some(ref session) = result.data {
                self.current_session = Some(session.clone());
            }
        }

        Ok(result)
    }

    pub async fn upload_certificate(&self, certificate_data: String, device_info: String, method: String) -> Result<ApiResponse<Certificate>, Box<dyn std::error::Error>> {
        if let Some(ref session) = self.current_session {
            let url = format!("{}/api/certificates", self.server_url);
            
            let request = UploadCertificateRequest {
                certificate_data,
                device_info,
                sanitization_method: method,
            };

            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", session.token))
                .json(&request)
                .send()
                .await?;

            let result: ApiResponse<Certificate> = response.json().await?;
            Ok(result)
        } else {
            Ok(ApiResponse {
                success: false,
                data: None,
                message: "Not authenticated. Please login first.".to_string(),
            })
        }
    }

    pub async fn get_user_certificates(&self) -> Result<ApiResponse<Vec<Certificate>>, Box<dyn std::error::Error>> {
        if let Some(ref session) = self.current_session {
            let url = format!("{}/api/certificates", self.server_url);

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", session.token))
                .send()
                .await?;

            let result: ApiResponse<Vec<Certificate>> = response.json().await?;
            Ok(result)
        } else {
            Ok(ApiResponse {
                success: false,
                data: None,
                message: "Not authenticated. Please login first.".to_string(),
            })
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.current_session.as_ref().map(|s| s.is_authenticated).unwrap_or(false)
    }

    pub fn get_current_user(&self) -> Option<&UserSession> {
        self.current_session.as_ref()
    }

    pub fn logout(&mut self) {
        self.current_session = None;
    }

    pub async fn test_connection(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let url = format!("{}/api/health", self.server_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCertificateRequest {
    pub certificate_data: String,
    pub device_info: String,
    pub sanitization_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub user_id: String,
    pub certificate_data: String,
    pub device_info: String,
    pub sanitization_method: String,
    pub created_at: String,
    pub file_hash: String,
}