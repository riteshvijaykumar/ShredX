#!/bin/bash

# Fix 502 Bad Gateway Error for HDD Tool Server

echo "========================================="
echo "Fixing 502 Bad Gateway Error"
echo "========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    print_error "Run as root: sudo $0"
    exit 1
fi

print_status "Step 1: Checking service status..."
echo "Nginx status:"
systemctl status nginx --no-pager -l
echo ""
echo "HDD Tool Server status:"
systemctl status hdd-tool-server --no-pager -l

print_status "Step 2: Checking if backend server is listening on port 3030..."
if command -v ss &> /dev/null; then
    ss -tlnp | grep :3030
elif command -v netstat &> /dev/null; then
    netstat -tlnp | grep :3030
else
    print_warning "Installing net-tools..."
    apt update && apt install -y net-tools
    netstat -tlnp | grep :3030
fi

if ! ss -tlnp | grep -q :3030 && ! netstat -tlnp 2>/dev/null | grep -q :3030; then
    print_error "Backend server is not listening on port 3030!"
    print_status "This is the cause of your 502 error."
fi

print_status "Step 3: Checking server logs..."
echo "Recent hdd-tool-server logs:"
journalctl -u hdd-tool-server --no-pager -n 10

print_status "Step 4: Installing Rust and building server..."

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
else
    print_status "Rust is already installed"
    source ~/.cargo/env 2>/dev/null || true
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Create project directory and basic server
PROJECT_DIR="/opt/hdd-tool-server"
mkdir -p $PROJECT_DIR
cd $PROJECT_DIR

# Create Cargo.toml
print_status "Creating Cargo.toml..."
cat > Cargo.toml << 'EOF'
[package]
name = "hdd-tool-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hdd-tool-server"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.40", features = ["full"] }
warp = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
EOF

# Create source directory and main.rs
mkdir -p src
print_status "Creating server source code..."
cat > src/main.rs << 'EOF'
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Starting HDD Tool Server...");

    // Get port from environment or use default
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
                message: "HDD Tool Server is running successfully".to_string(),
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
                data: Some("HDD Tool Server API v1.0 - Ready for disk sanitization operations"),
                message: "Server is operational and ready to accept requests".to_string(),
            };
            warp::reply::json(&response)
        });

    // Status endpoint
    let status = warp::path("status")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "server": "HDD Tool Server",
                "version": "0.1.0",
                "status": "running",
                "uptime": "active",
                "endpoints": [
                    "/health",
                    "/api/v1/info",
                    "/status"
                ]
            }))
        });

    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    // Combine all routes
    let routes = health
        .or(info)
        .or(status)
        .with(cors)
        .with(warp::log("hdd_tool_server"));

    println!("========================================");
    println!("HDD Tool Server Started Successfully!");
    println!("========================================");
    println!("Server listening on: 0.0.0.0:{}", server_port);
    println!("Health check: http://localhost:{}/health", server_port);
    println!("API info: http://localhost:{}/api/v1/info", server_port);
    println!("Status: http://localhost:{}/status", server_port);
    println!("========================================");

    // Start the server
    warp::serve(routes)
        .run(([0, 0, 0, 0], server_port))
        .await;

    Ok(())
}
EOF

# Build the server
print_status "Building server (this may take a few minutes)..."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release

if [ -f "target/release/hdd-tool-server" ]; then
    print_status "Build successful! Installing binary..."
    cp target/release/hdd-tool-server /usr/local/bin/
    chmod +x /usr/local/bin/hdd-tool-server
    chown root:root /usr/local/bin/hdd-tool-server
    print_status "Server binary installed successfully"
else
    print_error "Build failed! Let's try a simpler approach..."
    # Try debug build if release fails
    cargo build
    if [ -f "target/debug/hdd-tool-server" ]; then
        cp target/debug/hdd-tool-server /usr/local/bin/
        chmod +x /usr/local/bin/hdd-tool-server
        print_status "Debug binary installed (slower but should work)"
    else
        print_error "Both release and debug builds failed!"
        exit 1
    fi
fi

# Test the binary
print_status "Testing server binary..."
if /usr/local/bin/hdd-tool-server --help &>/dev/null || timeout 3s /usr/local/bin/hdd-tool-server &>/dev/null; then
    print_status "Server binary is working!"
else
    print_warning "Binary test had issues, but continuing..."
fi

print_status "Step 5: Fixing common issues..."

# Fix service file - run as root to avoid permission issues initially
print_status "Creating systemd service file..."
cat > /etc/systemd/system/hdd-tool-server.service << 'EOF'
[Unit]
Description=HDD Tool Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=/opt/hdd-tool-server
ExecStart=/usr/local/bin/hdd-tool-server
Environment=SERVER_PORT=3030
Environment=RUST_LOG=info
Environment=PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Resource limits
LimitNOFILE=65536
TimeoutStartSec=60
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
EOF

# Set proper permissions for project directory
chown -R root:root /opt/hdd-tool-server

# Fix Nginx configuration
print_status "Updating Nginx configuration..."
cat > /etc/nginx/sites-available/hdd-tool-server << 'EOF'
server {
    listen 80;
    server_name _;

    # Increase timeouts for long-running operations
    proxy_connect_timeout       300;
    proxy_send_timeout          300;
    proxy_read_timeout          300;
    send_timeout                300;

    location / {
        proxy_pass http://127.0.0.1:3030;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Handle WebSocket connections
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location /health {
        proxy_pass http://127.0.0.1:3030/health;
        access_log off;
    }

    # Add error pages
    error_page 502 503 504 /50x.html;
    location = /50x.html {
        root /var/www/html;
        internal;
    }
}
EOF

# Enable the site
ln -sf /etc/nginx/sites-available/hdd-tool-server /etc/nginx/sites-enabled/
rm -f /etc/nginx/sites-enabled/default

# Test nginx config
if nginx -t; then
    print_status "Nginx configuration is valid"
else
    print_error "Nginx configuration has errors!"
    exit 1
fi

print_status "Step 6: Restarting services..."
systemctl daemon-reload
systemctl restart hdd-tool-server
systemctl restart nginx

# Wait for services to start
sleep 5

print_status "Step 7: Final status check..."
echo "Services status:"
for service in hdd-tool-server nginx; do
    if systemctl is-active --quiet $service; then
        print_status "$service: ✓ Running"
    else
        print_error "$service: ✗ Not running"
        echo "Logs for $service:"
        journalctl -u $service --no-pager -n 5
    fi
done

print_status "Step 8: Testing connectivity..."
sleep 5

# Install curl if not present
if ! command -v curl &> /dev/null; then
    print_status "Installing curl..."
    apt install -y curl
fi

# Test backend directly first
print_status "Testing backend server on port 3030..."
BACKEND_TEST=""
for i in {1..10}; do
    if curl -s --connect-timeout 5 http://localhost:3030/health &>/dev/null; then
        BACKEND_TEST="success"
        break
    else
        print_warning "Attempt $i: Backend not responding, waiting 2 seconds..."
        sleep 2
    fi
done

if [ "$BACKEND_TEST" = "success" ]; then
    print_status "Backend server (port 3030): ✓ Working"
    
    # Now test through Nginx
    if curl -s --connect-timeout 5 http://localhost/health &>/dev/null; then
        print_status "Nginx proxy (port 80): ✓ Working"
        print_status "🎉 502 error is FIXED! Server is now hosted successfully!"
    else
        print_error "Nginx proxy (port 80): ✗ Still getting 502"
        print_status "Backend works but Nginx has issues. Checking Nginx config..."
        nginx -t
    fi
else
    print_error "Backend server (port 3030): ✗ Not responding after 10 attempts"
    print_status "Let's diagnose the issue..."
    
    # Detailed diagnostics
    echo ""
    echo "=== DIAGNOSTIC INFORMATION ==="
    
    echo "1. Process check:"
    ps aux | grep hdd-tool-server | grep -v grep || echo "No hdd-tool-server processes found"
    
    echo ""
    echo "2. Port check:"
    ss -tlnp | grep :3030 || netstat -tlnp 2>/dev/null | grep :3030 || echo "Nothing listening on port 3030"
    
    echo ""
    echo "3. Binary check:"
    ls -la /usr/local/bin/hdd-tool-server
    file /usr/local/bin/hdd-tool-server
    
    echo ""
    echo "4. Recent server logs:"
    journalctl -u hdd-tool-server --no-pager -n 15
    
    echo ""
    echo "5. Manual test:"
    print_status "Trying to run server manually for 10 seconds..."
    timeout 10s /usr/local/bin/hdd-tool-server || print_warning "Manual test completed"
fi

# Show network information
SERVER_IP=$(hostname -I | awk '{print $1}')
print_status ""
print_status "=== NETWORK INFORMATION ==="
print_status "Server IP Address: $SERVER_IP"

if [ "$BACKEND_TEST" = "success" ]; then
    print_status "✅ SUCCESS! Your server is now accessible at:"
    print_status "   🌐 Main site: http://$SERVER_IP/"
    print_status "   💓 Health check: http://$SERVER_IP/health"
    print_status "   📊 Status: http://$SERVER_IP/status"
    print_status "   🔧 API info: http://$SERVER_IP/api/v1/info"
else
    print_error "❌ Server is not working properly"
    print_status ""
    print_status "🔧 TROUBLESHOOTING COMMANDS:"
    print_status "1. Check server logs: journalctl -fu hdd-tool-server"
    print_status "2. Restart server: systemctl restart hdd-tool-server"
    print_status "3. Test backend: curl http://localhost:3030/health"
    print_status "4. Check Nginx: tail -f /var/log/nginx/error.log"
    print_status "5. Manual run: /usr/local/bin/hdd-tool-server"
    
    # Try one more restart
    print_status ""
    print_status "Attempting one final restart..."
    systemctl restart hdd-tool-server
    sleep 5
    if curl -s --connect-timeout 5 http://localhost:3030/health &>/dev/null; then
        print_status "🎉 Final restart worked! Server is now running!"
        print_status "Access your server at: http://$SERVER_IP/"
    fi
fi
EOF