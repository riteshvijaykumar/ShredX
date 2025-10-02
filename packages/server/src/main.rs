use std::sync::Arc;
use warp::{Filter, http::Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::{info, error};
use tera::Tera;

mod api;
mod auth;
mod database;

use api::routes::create_api_routes;
use auth::middleware::with_auth;
use database::init::init_database;

#[derive(Debug, Serialize, Deserialize)]
struct AppState {
    pool: Arc<PgPool>,
    templates: Arc<Tera>,
}

async fn serve_static() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::fs::dir("static"))
}

async fn serve_index(templates: Arc<Tera>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut context = tera::Context::new();
    context.insert("title", "HDD Tool Server");
    
    match templates.render("index.html", &context) {
        Ok(rendered) => Ok(Response::builder()
            .header("content-type", "text/html")
            .body(rendered)
            .unwrap()),
        Err(e) => {
            error!("Template rendering error: {}", e);
            Ok(Response::builder()
                .status(500)
                .body("Internal Server Error".to_string())
                .unwrap())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::init();
    
    info!("🚀 Starting HDD Tool Server...");
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://hdd_user:root@localhost/hdd_tool_db".to_string());
    
    let pool = Arc::new(PgPool::connect(&database_url).await?);
    info!("📊 Database: Connected successfully");
    
    // Initialize database schema
    init_database(pool.clone()).await?;
    
    // Initialize templates
    let templates = Arc::new(Tera::new("templates/**/*")?);
    info!("📄 Templates: Initialized successfully");
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization", "x-requested-with"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
    
    // Static files route
    let static_files = warp::path("static")
        .and(warp::fs::dir("static"));
    
    // Index route
    let index = warp::path::end()
        .and(warp::get())
        .and(warp::any().map(move || templates.clone()))
        .and_then(serve_index);
    
    // API routes
    let api_routes = create_api_routes(pool.clone());
    
    // Combine all routes
    let routes = index
        .or(static_files)
        .or(api_routes)
        .with(cors)
        .recover(handle_rejection);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;
    
    info!("🌐 Server starting on http://0.0.0.0:{}", port);
    info!("🎯 Web interface: http://localhost:{}", port);
    info!("🔗 API endpoint: http://localhost:{}/api", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, std::convert::Infallible> {
    let code;
    let message;
    
    if err.is_not_found() {
        code = warp::http::StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        code = warp::http::StatusCode::BAD_REQUEST;
        message = "Invalid Body";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
        message = "Method Not Allowed";
    } else {
        error!("Unhandled rejection: {:?}", err);
        code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }
    
    let json = warp::reply::json(&serde_json::json!({
        "error": message,
        "code": code.as_u16()
    }));
    
    Ok(warp::reply::with_status(json, code))
}