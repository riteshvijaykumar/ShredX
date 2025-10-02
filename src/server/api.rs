use warp::{Filter, Reply};
use std::sync::Arc;
use uuid::Uuid;
use crate::server::{DatabaseManager, models::*};
use sha2::{Sha256, Digest};

pub async fn start_server(database_url: String, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = Arc::new(DatabaseManager::new(&database_url).await?);
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);
    
    // Static files route for dashboard
    let dashboard = warp::path::end()
        .and(warp::get())
        .and_then(serve_dashboard);

    // Routes
    let register = warp::path("api")
        .and(warp::path("auth"))
        .and(warp::path("register"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(register_user);
    
    let login = warp::path("api")
        .and(warp::path("auth"))
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(login_user);
    
    let submit_cert = warp::path("api")
        .and(warp::path("certificates"))
        .and(warp::post())
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(submit_certificate);
    
    let get_certs = warp::path("api")
        .and(warp::path("certificates"))
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<PaginationQuery>())
        .and(with_db(db.clone()))
        .and_then(get_certificates);
    
    let get_logs = warp::path("api")
        .and(warp::path("logs"))
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<PaginationQuery>())
        .and(with_db(db.clone()))
        .and_then(get_sanitization_logs);
    
    // Certificate download route
    let download_cert = warp::path("api")
        .and(warp::path("certificates"))
        .and(warp::path::param::<Uuid>())
        .and(warp::path("download"))
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(with_db(db.clone()))
        .and_then(download_certificate);
    
    let routes = dashboard
        .or(register)
        .or(login)
        .or(submit_cert)
        .or(get_certs)
        .or(download_cert)
        .or(get_logs)
        .with(cors);
    
    println!("🚀 HDD Tool Server starting on port {}", port);
    println!("📊 Dashboard available at: http://localhost:{}/", port);
    println!("🔗 API endpoints:");
    println!("   POST /api/auth/register - Create user account");
    println!("   POST /api/auth/login - User login");
    println!("   POST /api/certificates - Submit certificate");
    println!("   GET  /api/certificates - Get user certificates");
    println!("   GET  /api/certificates/:id/download - Download certificate");
    println!("   GET  /api/logs - Get sanitization logs");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
        
    Ok(())
}

fn with_db(db: Arc<DatabaseManager>) -> impl Filter<Extract = (Arc<DatabaseManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

#[derive(serde::Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 { 50 }

// Extract user ID from Bearer token (simplified - in production use JWT)
fn extract_user_id(auth_header: &str) -> Result<Uuid, String> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Uuid::parse_str(token).map_err(|_| "Invalid token format".to_string())
    } else {
        Err("Invalid authorization header".to_string())
    }
}

async fn register_user(
    req: CreateUserRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.create_user(req).await {
        Ok(user) => {
            let response = ApiResponse::success(LoginResponse {
                token: user.id.to_string(), // Simplified - use JWT in production
                user_id: user.id,
                username: user.username,
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(format!("Registration failed: {}", e));
            Ok(warp::reply::json(&response))
        }
    }
}

async fn login_user(
    req: LoginRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.authenticate_user(req).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(LoginResponse {
                token: user.id.to_string(), // Simplified - use JWT in production
                user_id: user.id,
                username: user.username,
            });
            Ok(warp::reply::json(&response))
        }
        Ok(None) => {
            let response: ApiResponse<()> = ApiResponse::error("Invalid credentials".to_string());
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(format!("Login failed: {}", e));
            Ok(warp::reply::json(&response))
        }
    }
}

async fn submit_certificate(
    auth_header: String,
    req: SubmitCertificateRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            let file_hash = format!("{:x}", Sha256::digest(req.certificate_data.as_bytes()));
            let store_req = StoreCertificateRequest {
                user_id,
                certificate_data: req.certificate_data,
                device_info: req.device_info,
                sanitization_method: req.sanitization_method,
                file_hash,
            };
            match db.store_certificate(store_req).await {
                Ok(certificate) => {
                    let response = ApiResponse::success(certificate);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to store certificate: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}

async fn get_certificates(
    auth_header: String,
    query: PaginationQuery,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.get_user_certificates(user_id, query.limit, query.offset).await {
                Ok(certificates) => {
                    let response = ApiResponse::success(certificates);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to get certificates: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}

async fn get_sanitization_logs(
    auth_header: String,
    query: PaginationQuery,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.get_sanitization_logs(user_id, query.limit, query.offset).await {
                Ok(logs) => {
                    let response = ApiResponse::success(logs);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to get logs: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}

async fn serve_dashboard() -> Result<impl warp::Reply, warp::Rejection> {
    let dashboard_html = include_str!("dashboard.html");
    Ok(warp::reply::html(dashboard_html))
}

async fn download_certificate(
    cert_id: Uuid,
    auth_header: String,
    db: Arc<DatabaseManager>,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.get_certificate_by_id(cert_id, user_id).await {
                Ok(Some(certificate)) => {
                    let filename = format!("certificate_{}.json", cert_id);
                    
                    Ok(Box::new(warp::reply::with_header(
                        warp::reply::with_header(
                            certificate.certificate_data,
                            "content-disposition",
                            format!("attachment; filename={}", filename)
                        ),
                        "content-type",
                        "application/json"
                    )))
                }
                Ok(None) => {
                    Ok(Box::new(warp::reply::with_status(
                        "Certificate not found".to_string(),
                        warp::http::StatusCode::NOT_FOUND
                    )))
                }
                Err(e) => {
                    Ok(Box::new(warp::reply::with_status(
                        format!("Database error: {}", e),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR
                    )))
                }
            }
        }
        Err(e) => {
            Ok(Box::new(warp::reply::with_status(
                e,
                warp::http::StatusCode::UNAUTHORIZED
            )))
        }
    }
}