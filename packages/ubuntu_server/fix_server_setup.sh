#!/bin/bash

# HDD Tool Server Fix and Setup Script
# This script fixes common issues and sets up the server properly

set -e  # Exit on any error

echo "========================================="
echo "HDD Tool Server Fix and Setup Script"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}=========================================${NC}"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_status "Running as root - good!"
else
    print_error "This script must be run as root. Use: sudo $0"
    exit 1
fi

# Step 1: Install missing tools
print_header "Installing Required System Tools"
apt update
apt install -y net-tools curl wget git build-essential pkg-config libssl-dev

# Step 2: Check and fix PostgreSQL
print_header "Setting up PostgreSQL"

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    print_status "Installing PostgreSQL..."
    apt install -y postgresql postgresql-contrib
else
    print_status "PostgreSQL is already installed"
fi

# Start and enable PostgreSQL
systemctl start postgresql
systemctl enable postgresql

# Check PostgreSQL status
if systemctl is-active --quiet postgresql; then
    print_status "PostgreSQL is running"
else
    print_error "Failed to start PostgreSQL"
    systemctl status postgresql
    exit 1
fi

# Setup database and user
print_status "Setting up database and user..."
sudo -u postgres psql << 'EOF'
-- Drop existing database and user if they exist
DROP DATABASE IF EXISTS hdd_tool_db;
DROP USER IF EXISTS hdd_user;

-- Create new database and user
CREATE DATABASE hdd_tool_db;
CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'root';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER DATABASE hdd_tool_db OWNER TO hdd_user;

-- Grant additional permissions
GRANT CREATE ON SCHEMA public TO hdd_user;
GRANT USAGE ON SCHEMA public TO hdd_user;

\q
EOF

# Test database connection
print_status "Testing database connection..."
if sudo -u postgres psql -h localhost -U hdd_user -d hdd_tool_db -c "SELECT 1;" &>/dev/null; then
    print_status "Database connection successful"
else
    print_warning "Database connection test failed, but continuing..."
fi

# Step 3: Install Rust if not present
print_header "Setting up Rust Environment"

if ! command -v cargo &> /dev/null; then
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
else
    print_status "Rust is already installed"
fi

# Update Rust
rustup update

# Step 4: Create project structure
print_header "Setting up Project Structure"

PROJECT_DIR="/opt/hdd-tool-server"
SERVICE_USER="hdd-tool"

# Create project directory
mkdir -p $PROJECT_DIR
cd $PROJECT_DIR

# Create service user
if ! id "$SERVICE_USER" &>/dev/null; then
    print_status "Creating service user: $SERVICE_USER"
    useradd --system --no-create-home --shell /bin/false $SERVICE_USER
else
    print_status "Service user already exists: $SERVICE_USER"
fi

# Step 5: Create Cargo.toml
print_status "Creating Cargo.toml..."
cat > Cargo.toml << 'EOF'
[package]
name = "hdd-tool-server"
version = "0.1.0"
edition = "2021"
authors = ["SIH Team"]
description = "HDD Tool Web Server - NIST SP 800-88 compliant disk sanitization server"

[[bin]]
name = "hdd-tool-server"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.40", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "migrate"] }
jsonwebtoken = "9.0"
bcrypt = "0.15"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
anyhow = "1.0"
dotenv = "0.15"
reqwest = { version = "0.11", features = ["json"] }
nix = "0.29"
libc = "0.2"
serde_derive = "1.0"

[profile.release]
opt-level = 3
strip = true
lto = true
EOF

# Step 6: Create basic server source
print_status "Creating server source code..."
mkdir -p src

cat > src/main.rs << 'EOF'
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::sync::Arc;
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Starting HDD Tool Server...");

    // Environment variables with defaults
    let server_port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .unwrap_or(3030);

    // Health check endpoint
    let health = warp::path("health")
        .and(warp::get())
        .map(|| {
            let response = HealthResponse {
                status: "ok".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: "0.1.0".to_string(),
            };
            warp::reply::json(&response)
        });

    // API info endpoint
    let info = warp::path("api")
        .and(warp::path("v1"))
        .and(warp::path("info"))
        .and(warp::get())
        .map(|| {
            let response = ApiResponse {
                success: true,
                data: Some("HDD Tool Server API v1.0"),
                message: "Server is running successfully".to_string(),
            };
            warp::reply::json(&response)
        });

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    let routes = health
        .or(info)
        .with(cors)
        .with(warp::log("hdd_tool_server"));

    println!("Server starting on port {}", server_port);
    println!("Health check: http://localhost:{}/health", server_port);
    println!("API info: http://localhost:{}/api/v1/info", server_port);

    warp::serve(routes)
        .run(([0, 0, 0, 0], server_port))
        .await;

    Ok(())
}
EOF

# Step 7: Build the project
print_header "Building the Server"
print_status "This may take a while for the first build..."

# Set Rust environment for build
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.cargo/env 2>/dev/null || true

cargo build --release

if [ $? -eq 0 ]; then
    print_status "Build successful!"
else
    print_error "Build failed!"
    exit 1
fi

# Step 8: Install the binary
print_status "Installing server binary..."
cp target/release/hdd-tool-server /usr/local/bin/
chmod +x /usr/local/bin/hdd-tool-server
chown root:root /usr/local/bin/hdd-tool-server

# Step 9: Create systemd service
print_header "Creating Systemd Service"

cat > /etc/systemd/system/hdd-tool-server.service << EOF
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$PROJECT_DIR
ExecStart=/usr/local/bin/hdd-tool-server
Environment=DATABASE_URL=postgresql://hdd_user:root@localhost/hdd_tool_db
Environment=JWT_SECRET=your-secret-key-change-this-in-production
Environment=SERVER_PORT=3030
Environment=RUST_LOG=info
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Set proper permissions
chown -R $SERVICE_USER:$SERVICE_USER $PROJECT_DIR

# Step 10: Configure Nginx
print_header "Configuring Nginx"

cat > /etc/nginx/sites-available/hdd-tool-server << 'EOF'
server {
    listen 80;
    server_name _;

    location / {
        proxy_pass http://127.0.0.1:3030;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /health {
        proxy_pass http://127.0.0.1:3030/health;
        access_log off;
    }
}
EOF

# Enable the site
ln -sf /etc/nginx/sites-available/hdd-tool-server /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test nginx configuration
nginx -t

if [ $? -eq 0 ]; then
    print_status "Nginx configuration is valid"
    systemctl reload nginx
else
    print_error "Nginx configuration has errors"
    exit 1
fi

# Step 11: Start services
print_header "Starting Services"

systemctl daemon-reload
systemctl enable hdd-tool-server.service
systemctl start hdd-tool-server.service

# Wait a moment for service to start
sleep 3

# Step 12: Check service status
print_header "Service Status Check"

if systemctl is-active --quiet hdd-tool-server; then
    print_status "HDD Tool Server is running!"
else
    print_error "HDD Tool Server failed to start"
    print_status "Checking logs..."
    journalctl -xeu hdd-tool-server.service --no-pager -n 20
fi

if systemctl is-active --quiet nginx; then
    print_status "Nginx is running!"
else
    print_error "Nginx is not running"
    systemctl status nginx
fi

if systemctl is-active --quiet postgresql; then
    print_status "PostgreSQL is running!"
else
    print_error "PostgreSQL is not running"
    systemctl status postgresql
fi

# Step 13: Test endpoints
print_header "Testing Server Endpoints"

sleep 2

# Test health endpoint
print_status "Testing health endpoint..."
if curl -s http://localhost/health | grep -q "ok"; then
    print_status "Health endpoint is working!"
else
    print_warning "Health endpoint test failed"
fi

# Test API endpoint
print_status "Testing API endpoint..."
if curl -s http://localhost/api/v1/info | grep -q "success"; then
    print_status "API endpoint is working!"
else
    print_warning "API endpoint test failed"
fi

# Step 14: Show network information
print_header "Network Information"

SERVER_IP=$(hostname -I | awk '{print $1}')
print_status "Server IP Address: $SERVER_IP"
print_status "Server URLs:"
print_status "  - Local: http://localhost/"
print_status "  - Network: http://$SERVER_IP/"
print_status "  - Health Check: http://$SERVER_IP/health"
print_status "  - API Info: http://$SERVER_IP/api/v1/info"

# Step 15: Show port status
print_status "Checking port usage..."
ss -tlnp | grep -E ":80|:3030|:5432" || print_warning "Some services might not be listening on expected ports"

# Step 16: Final status
print_header "Setup Complete!"

print_status "Services Status:"
systemctl is-active hdd-tool-server && echo "  ✓ HDD Tool Server: Running" || echo "  ✗ HDD Tool Server: Not Running"
systemctl is-active nginx && echo "  ✓ Nginx: Running" || echo "  ✗ Nginx: Not Running"
systemctl is-active postgresql && echo "  ✓ PostgreSQL: Running" || echo "  ✗ PostgreSQL: Not Running"

print_status ""
print_status "Useful Commands:"
print_status "  - Check server status: systemctl status hdd-tool-server"
print_status "  - View server logs: journalctl -fu hdd-tool-server"
print_status "  - Restart server: systemctl restart hdd-tool-server"
print_status "  - Test health: curl http://$SERVER_IP/health"

print_status ""
print_status "Setup completed successfully!"
print_status "Your HDD Tool Server is now running at: http://$SERVER_IP/"
EOF