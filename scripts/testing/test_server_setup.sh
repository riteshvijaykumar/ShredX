#!/bin/bash

# Simple test script to verify the server setup steps work
# Run this first to test the critical parts

set -e

echo "🧪 Testing HDD Tool Server Setup Components..."
echo ""

# Test 1: Check if we're in the right directory
echo "📂 Current directory: $(pwd)"
echo "📂 Creating test directory..."
mkdir -p /tmp/hdd-tool-test
cd /tmp/hdd-tool-test

# Test 2: Test Rust environment
echo "🦀 Testing Rust..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust found: $(rustc --version)"
else
    echo "❌ Rust not found - need to install"
    exit 1
fi

# Test 3: Test PostgreSQL connection
echo "🐘 Testing PostgreSQL connection..."
if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;' &>/dev/null; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed"
    echo "   Make sure PostgreSQL is set up with:"
    echo "   - Database: hdd_tool_db"
    echo "   - User: hdd_user" 
    echo "   - Password: root"
    exit 1
fi

# Test 4: Create basic Cargo project structure
echo "📦 Testing Cargo project creation..."
mkdir -p src/server

cat > Cargo.toml << 'EOF'
[package]
name = "hdd_tool_server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hdd-tool-server"
path = "src/server/main.rs"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
warp = "0.3"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonwebtoken = "9.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
sha2 = "0.10"
base64 = "0.21"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
hex = "0.4"
EOF

echo "✅ Cargo.toml created"

# Test 5: Create minimal server
echo "🖥️ Creating minimal server..."
cat > src/server/main.rs << 'EOF'
use std::sync::Arc;
use warp::Filter;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use sha2::{Sha256, Digest};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    email: String,
    role: String,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn health_handler() -> Result<impl warp::Reply, warp::Rejection> {
    let response = ApiResponse {
        success: true,
        data: Some("HDD Tool Server is running"),
        message: "Server healthy".to_string(),
    };
    Ok(warp::reply::json(&response))
}

async fn login_handler(
    body: LoginRequest,
    pool: Arc<PgPool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let password_hash = hash_password(&body.password);
    
    let user_row = sqlx::query(
        "SELECT id, username, email, role, created_at, last_login, is_active 
         FROM users WHERE username = $1 AND password_hash = $2 AND is_active = true"
    )
    .bind(&body.username)
    .bind(&password_hash)
    .fetch_optional(pool.as_ref())
    .await;

    match user_row {
        Ok(Some(row)) => {
            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                role: row.get("role"),
                created_at: row.get("created_at"),
                last_login: row.get("last_login"),
                is_active: row.get("is_active"),
            };

            let token = format!("jwt_token_for_{}", user.username);

            let response = ApiResponse {
                success: true,
                data: Some(LoginResponse { token, user }),
                message: "Login successful".to_string(),
            };

            Ok(warp::reply::json(&response))
        }
        Ok(None) => {
            let response: ApiResponse<()> = ApiResponse {
                success: false,
                data: None,
                message: "Invalid credentials".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            let response: ApiResponse<()> = ApiResponse {
                success: false,
                data: None,
                message: "Server error".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HDD Tool Server Test starting...");
    
    // Database connection
    let database_url = "postgresql://hdd_user:root@localhost/hdd_tool_db";
    let pool = Arc::new(PgPool::connect(database_url).await?);
    
    println!("📊 Database: Connected successfully");
    println!("🌐 Server URL: http://0.0.0.0:3000");

    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // Routes
    let health = warp::path("api")
        .and(warp::path("health"))
        .and(warp::get())
        .and_then(health_handler);

    let login = warp::path("api")
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and_then(login_handler);

    let routes = health.or(login).with(cors);

    println!("✅ Starting server on port 3000...");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3000))
        .await;

    Ok(())
}
EOF

echo "✅ Server code created"

# Test 6: Try to build
echo "🔨 Testing build..."
source ~/.cargo/env
if cargo build --release --bin hdd-tool-server; then
    echo "✅ Build successful!"
    echo "📁 Binary location: $(pwd)/target/release/hdd-tool-server"
    ls -la target/release/hdd-tool-server
else
    echo "❌ Build failed"
    exit 1
fi

echo ""
echo "🎉 All tests passed! The server setup should work."
echo "📋 Next steps:"
echo "   1. Run this test in your Ubuntu server SSH session"
echo "   2. If it passes, run the full server_setup.sh script"
echo "   3. The server will be available at http://your-server-ip:3000"

# Cleanup
cd /
rm -rf /tmp/hdd-tool-test

echo "✅ Test completed successfully!"