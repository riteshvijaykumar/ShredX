#!/bin/bash

# HDD Tool Web Server - Complete Setup Script
# This script sets up everything needed for the web interface on Ubuntu

set -e

echo "🚀 HDD Tool Web Server Setup"
echo "============================="
echo "This will install and configure the complete web interface"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root for security reasons"
   print_error "Please run as a regular user with sudo privileges"
   exit 1
fi

# Get server info
SERVER_IP=$(hostname -I | awk '{print $1}')
USERNAME=$(whoami)

print_step "Starting setup for user: $USERNAME"
print_step "Detected server IP: $SERVER_IP"
echo ""

# Confirmation
read -p "Continue with setup? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Setup cancelled."
    exit 1
fi

# Step 1: Update system
print_step "1. Updating system packages..."
sudo apt update
sudo apt upgrade -y
print_success "System updated"

# Step 2: Install dependencies
print_step "2. Installing dependencies..."
sudo apt install -y \
    curl wget git unzip \
    build-essential pkg-config libssl-dev \
    postgresql postgresql-contrib \
    nginx \
    ufw
print_success "Dependencies installed"

# Step 3: Install Rust
print_step "3. Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed: $(rustc --version)"
else
    print_warning "Rust already installed: $(rustc --version)"
fi

# Step 4: Setup PostgreSQL
print_step "4. Setting up PostgreSQL..."
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << 'EOF'
DROP DATABASE IF EXISTS hdd_tool_db;
DROP USER IF EXISTS hdd_user;
CREATE DATABASE hdd_tool_db;
CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'root';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER DATABASE hdd_tool_db OWNER TO hdd_user;
\q
EOF

# Test database connection
if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;' &>/dev/null; then
    print_success "Database setup completed"
else
    print_error "Database setup failed"
    exit 1
fi

# Step 5: Create project directory
print_step "5. Creating project structure..."
PROJECT_DIR="/home/$USERNAME/hdd-tool-server"
mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

# Create directory structure
mkdir -p src/{api,auth,database}
mkdir -p static/{css,js}
mkdir -p templates
mkdir -p config

print_success "Project structure created at $PROJECT_DIR"

# Step 6: Create server files
print_step "6. Creating server configuration..."

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "hdd_tool_server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hdd-tool-server"
path = "src/main.rs"

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
tera = "1.19"
EOF

# Create environment file
cat > .env << EOF
DATABASE_URL=postgresql://hdd_user:root@localhost/hdd_tool_db
PORT=3000
HOST=0.0.0.0
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

print_success "Configuration files created"

# Step 7: Create minimal working server
print_step "7. Creating server application..."
cat > src/main.rs << 'EOF'
use std::sync::Arc;
use warp::{Filter, http::Response};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use tracing::{info, error};

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

async fn index_handler() -> Result<impl warp::Reply, warp::Rejection> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>HDD Tool Server</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; text-align: center; }
        .status { background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 12px; border-radius: 4px; margin: 20px 0; }
        .info { background: #d1ecf1; border: 1px solid #bee5eb; color: #0c5460; padding: 12px; border-radius: 4px; margin: 20px 0; }
        .links { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0; }
        .link-box { background: #f8f9fa; border: 1px solid #dee2e6; padding: 15px; border-radius: 4px; text-decoration: none; color: #333; }
        .link-box:hover { background: #e9ecef; }
        code { background: #f8f9fa; padding: 2px 4px; border-radius: 3px; font-family: monospace; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🛠️ HDD Tool Server</h1>
        
        <div class="status">
            ✅ Server is running successfully!
        </div>
        
        <div class="info">
            <strong>Server Information:</strong><br>
            • Web Interface: <code>http://SERVER_IP</code><br>
            • API Endpoint: <code>http://SERVER_IP/api</code><br>
            • Health Check: <code>http://SERVER_IP/api/health</code>
        </div>
        
        <div class="links">
            <a href="/api/health" class="link-box">
                <strong>🔍 Health Check</strong><br>
                Test server status
            </a>
            <a href="/api/login" class="link-box">
                <strong>🔐 Login API</strong><br>
                Authentication endpoint
            </a>
        </div>
        
        <div class="info">
            <strong>Default Credentials:</strong><br>
            • Admin: <code>admin</code> / <code>admin123</code><br>
            • User: <code>user</code> / <code>user123</code>
        </div>
        
        <div class="info">
            <strong>Next Steps:</strong><br>
            1. Configure your Windows client to connect to this server<br>
            2. Change default passwords for security<br>
            3. Start using the HDD Tool remotely!
        </div>
    </div>
</body>
</html>
    "#;
    
    Ok(Response::builder()
        .header("content-type", "text/html")
        .body(html.replace("SERVER_IP", &std::env::var("SERVER_IP").unwrap_or_else(|_| "YOUR_SERVER_IP".to_string())))
        .unwrap())
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
            error!("Database error: {}", e);
            let response: ApiResponse<()> = ApiResponse {
                success: false,
                data: None,
                message: "Server error".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn init_database(pool: Arc<PgPool>) -> Result<(), sqlx::Error> {
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

    // Insert default users
    let admin_password = hash_password("admin123");
    let user_password = hash_password("user123");
    
    sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind("admin")
    .bind("admin@hddtool.local")
    .bind(&admin_password)
    .bind("admin")
    .execute(pool.as_ref())
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users (username, email, password_hash, role)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (username) DO NOTHING
        "#,
    )
    .bind("user")
    .bind("user@hddtool.local")
    .bind(&user_password)
    .bind("user")
    .execute(pool.as_ref())
    .await?;

    info!("Database initialized successfully");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    info!("🚀 Starting HDD Tool Server...");
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://hdd_user:root@localhost/hdd_tool_db".to_string());
    
    let pool = Arc::new(PgPool::connect(&database_url).await?);
    info!("📊 Database connected successfully");
    
    // Initialize database
    init_database(pool.clone()).await?;
    
    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // Routes
    let index = warp::path::end()
        .and(warp::get())
        .and_then(index_handler);

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

    let routes = index.or(health).or(login).with(cors);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;
    
    info!("🌐 Server starting on http://0.0.0.0:{}", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}
EOF

print_success "Server application created"

# Step 8: Build the server
print_step "8. Building server (this may take a while)..."
source ~/.cargo/env
cargo build --release

if [ $? -eq 0 ]; then
    print_success "Server built successfully"
else
    print_error "Server build failed"
    exit 1
fi

# Step 9: Setup systemd service
print_step "9. Setting up system service..."
sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << EOF
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$USERNAME
WorkingDirectory=$PROJECT_DIR
Environment=PATH=/home/$USERNAME/.cargo/bin:\$PATH
Environment=SERVER_IP=$SERVER_IP
EnvironmentFile=$PROJECT_DIR/.env
ExecStart=$PROJECT_DIR/target/release/hdd-tool-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server
print_success "System service configured"

# Step 10: Setup Nginx
print_step "10. Setting up web server..."
sudo tee /etc/nginx/sites-available/hdd-tool << EOF
server {
    listen 80;
    server_name $SERVER_IP localhost;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
    }
}
EOF

sudo ln -sf /etc/nginx/sites-available/hdd-tool /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl restart nginx
sudo systemctl enable nginx
print_success "Web server configured"

# Step 11: Configure firewall
print_step "11. Configuring firewall..."
sudo ufw allow ssh
sudo ufw allow 80/tcp comment "HTTP"
sudo ufw allow 443/tcp comment "HTTPS"
sudo ufw --force enable
print_success "Firewall configured"

# Step 12: Start services
print_step "12. Starting services..."
sudo systemctl start hdd-tool-server
sleep 3

if sudo systemctl is-active --quiet hdd-tool-server; then
    print_success "HDD Tool Server started successfully"
else
    print_error "Failed to start HDD Tool Server"
    print_error "Check logs: sudo journalctl -u hdd-tool-server -f"
    exit 1
fi

# Step 13: Test the setup
print_step "13. Testing setup..."
sleep 5

if curl -s "http://localhost/api/health" | grep -q "healthy"; then
    print_success "Web interface is working!"
else
    print_warning "Web interface test failed, but server might still be starting..."
fi

# Final success message
echo ""
echo "🎉 Setup completed successfully!"
echo "================================"
echo ""
echo "📋 Server Information:"
echo "  🌐 Web Interface: http://$SERVER_IP"
echo "  🔗 API Endpoint: http://$SERVER_IP/api"
echo "  📊 Health Check: http://$SERVER_IP/api/health"
echo ""
echo "🔐 Default Credentials:"
echo "  Admin: admin / admin123"
echo "  User: user / user123"
echo ""
echo "🔧 Management Commands:"
echo "  Status: sudo systemctl status hdd-tool-server"
echo "  Logs:   sudo journalctl -u hdd-tool-server -f"
echo "  Restart: sudo systemctl restart hdd-tool-server"
echo ""
echo "📱 Windows Client Setup:"
echo "  1. Update your config.json:"
echo "     \"server_url\": \"http://$SERVER_IP\""
echo "  2. Set \"is_server_enabled\": true"
echo "  3. Start your HDD Tool application"
echo ""
print_success "Your HDD Tool web server is ready! 🚀"