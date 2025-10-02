#!/bin/bash

# HDD Tool Server Deployment Script
# Deploy from Windows client to Ubuntu server

set -e

# Configuration - CHANGE THESE VALUES
SERVER_IP="${1:-YOUR_SERVER_IP_HERE}"
SERVER_USER="${2:-ubuntu}"
PROJECT_NAME="hdd-tool-server"
REMOTE_DIR="/home/$SERVER_USER/$PROJECT_NAME"

echo "🚀 Deploying HDD Tool Server"
echo "============================="
echo "📡 Target: $SERVER_USER@$SERVER_IP"
echo "📁 Remote Directory: $REMOTE_DIR"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[DEPLOY]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validate inputs
if [ "$SERVER_IP" = "YOUR_SERVER_IP_HERE" ]; then
    print_error "Please provide the server IP address:"
    echo "Usage: $0 <server_ip> [username]"
    echo "Example: $0 192.168.1.100 ubuntu"
    exit 1
fi

# Test connection
print_status "Testing connection to $SERVER_USER@$SERVER_IP..."
if ! ssh -o ConnectTimeout=5 "$SERVER_USER@$SERVER_IP" "echo 'Connection successful'"; then
    print_error "Cannot connect to server. Please check:"
    echo "  - Server IP: $SERVER_IP"
    echo "  - Username: $SERVER_USER"
    echo "  - SSH key authentication"
    exit 1
fi

print_success "Server connection established"

# Create remote directory
print_status "Creating remote directory..."
ssh "$SERVER_USER@$SERVER_IP" "mkdir -p $REMOTE_DIR"

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

# Copy server files
print_status "Copying server files..."
rsync -avz --progress \
    --exclude target/ \
    --exclude .git/ \
    --exclude "*.log" \
    --exclude ubuntu_server/ \
    "$PROJECT_ROOT/server/" "$SERVER_USER@$SERVER_IP:$REMOTE_DIR/"

# Copy configuration files
print_status "Copying configuration files..."
scp "$SCRIPT_DIR/../config/server.env" "$SERVER_USER@$SERVER_IP:$REMOTE_DIR/.env"

print_success "Files copied successfully"

# Build on remote server
print_status "Building on remote server..."
ssh "$SERVER_USER@$SERVER_IP" << EOF
    cd $REMOTE_DIR
    
    # Source Rust environment
    source ~/.cargo/env 2>/dev/null || {
        echo "❌ Rust not found on server"
        exit 1
    }
    
    echo "🔨 Building project..."
    cargo build --release
    
    echo "✅ Build completed"
EOF

print_success "Build completed on server"

# Setup systemd service
print_status "Setting up systemd service..."
ssh "$SERVER_USER@$SERVER_IP" << EOF
    sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << 'SERVICE_EOF'
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$SERVER_USER
WorkingDirectory=$REMOTE_DIR
Environment=PATH=/home/$SERVER_USER/.cargo/bin:\$PATH
EnvironmentFile=$REMOTE_DIR/.env
ExecStart=$REMOTE_DIR/target/release/hdd-tool-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
SERVICE_EOF

    sudo systemctl daemon-reload
    sudo systemctl enable hdd-tool-server
EOF

print_success "Service configured"

# Start the service
print_status "Starting HDD Tool Server..."
ssh "$SERVER_USER@$SERVER_IP" << EOF
    sudo systemctl restart hdd-tool-server
    sleep 3
    
    if sudo systemctl is-active --quiet hdd-tool-server; then
        echo "✅ Service started successfully"
    else
        echo "❌ Service failed to start"
        sudo systemctl status hdd-tool-server --no-pager
        exit 1
    fi
EOF

# Test deployment
print_status "Testing deployment..."
sleep 5

if curl -s "http://$SERVER_IP/api/health" | grep -q "healthy"; then
    print_success "✅ Deployment successful!"
else
    print_error "❌ Deployment test failed"
    exit 1
fi

# Success message
print_success "🎉 Deployment completed successfully!"
echo ""
echo "📋 Deployment Summary:"
echo "====================="
echo "🌐 Web Interface: http://$SERVER_IP"
echo "🔗 API Endpoint: http://$SERVER_IP/api"
echo "📊 Health Check: http://$SERVER_IP/api/health"
echo "👤 SSH Access: ssh $SERVER_USER@$SERVER_IP"
echo "📁 Project Dir: $REMOTE_DIR"
echo ""
echo "🔧 Server Management:"
echo "  Status: ssh $SERVER_USER@$SERVER_IP 'sudo systemctl status hdd-tool-server'"
echo "  Logs:   ssh $SERVER_USER@$SERVER_IP 'sudo journalctl -u hdd-tool-server -f'"
echo "  Restart: ssh $SERVER_USER@$SERVER_IP 'sudo systemctl restart hdd-tool-server'"
echo ""
echo "🔐 Default Web Credentials:"
echo "  Admin: admin / admin123"
echo "  User: user / user123"