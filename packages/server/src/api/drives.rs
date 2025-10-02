use std::sync::Arc;
use warp::Filter;
use sqlx::PgPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Drive {
    id: String,
    name: String,
    size: u64,
    drive_type: String,
    model: Option<String>,
    serial: Option<String>,
    is_connected: bool,
    last_scan: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn create_drive_routes(pool: Arc<PgPool>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let list_drives = warp::path("api")
        .and(warp::path("drives"))
        .and(warp::get())
        .and(with_pool(pool.clone()))
        .and_then(list_drives_handler);
    
    let scan_drives = warp::path("api")
        .and(warp::path("drives"))
        .and(warp::path("scan"))
        .and(warp::post())
        .and(with_pool(pool.clone()))
        .and_then(scan_drives_handler);
    
    list_drives.or(scan_drives)
}

fn with_pool(pool: Arc<PgPool>) -> impl Filter<Extract = (Arc<PgPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn list_drives_handler(pool: Arc<PgPool>) -> Result<impl warp::Reply, warp::Rejection> {
    // In a real implementation, this would scan the system for drives
    // For now, return mock data
    let drives = vec![
        Drive {
            id: "sda".to_string(),
            name: "/dev/sda".to_string(),
            size: 1000000000000, // 1TB
            drive_type: "SSD".to_string(),
            model: Some("Samsung 980 PRO".to_string()),
            serial: Some("S1234567890".to_string()),
            is_connected: true,
            last_scan: Some(chrono::Utc::now()),
        },
        Drive {
            id: "sdb".to_string(),
            name: "/dev/sdb".to_string(),
            size: 2000000000000, // 2TB
            drive_type: "HDD".to_string(),
            model: Some("Seagate Barracuda".to_string()),
            serial: Some("ST2000DM008".to_string()),
            is_connected: true,
            last_scan: Some(chrono::Utc::now()),
        },
    ];
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "data": drives,
        "message": "Drives listed successfully"
    })))
}

async fn scan_drives_handler(_pool: Arc<PgPool>) -> Result<impl warp::Reply, warp::Rejection> {
    // In a real implementation, this would trigger a system drive scan
    // For now, just return success
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "message": "Drive scan initiated"
    })))
}