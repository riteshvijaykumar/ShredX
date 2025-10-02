#!/bin/bash

# HDD Tool Server Deployment Script
# This script deploys the server to an Ubuntu machine

set -e

# Configuration
SERVER_IP="${1:-localhost}"
SERVER_USER="${2:-ubuntu}"
PROJECT_NAME="hdd-tool-server"
REMOTE_DIR="/home/$SERVER_USER/$PROJECT_NAME"

echo "🚀 Deploying HDD Tool Server to $SERVER_USER@$SERVER_IP"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[DEPLOY]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we can connect to the server
print_status "Testing connection to $SERVER_USER@$SERVER_IP..."
if ! ssh -o ConnectTimeout=5 "$SERVER_USER@$SERVER_IP" "echo 'Connection successful'"; then
    print_error "Cannot connect to server. Please check:"
    echo "  - Server IP address: $SERVER_IP"
    echo "  - Username: $SERVER_USER"
    echo "  - SSH key authentication is set up"
    exit 1
fi

print_success "Server connection established"

# Create remote directory
print_status "Creating remote directory..."
ssh "$SERVER_USER@$SERVER_IP" "mkdir -p $REMOTE_DIR"

# Copy source files
print_status "Copying source files..."
rsync -avz --progress \
    --exclude target/ \
    --exclude .git/ \
    --exclude "*.log" \
    ./ "$SERVER_USER@$SERVER_IP:$REMOTE_DIR/"

print_success "Source files copied"

# Install dependencies and build on remote server
print_status "Building on remote server..."
ssh "$SERVER_USER@$SERVER_IP" << EOF
    cd $REMOTE_DIR
    
    # Source Rust environment
    source ~/.cargo/env 2>/dev/null || {
        echo "Rust not found, installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    }
    
    echo "Building project..."
    cargo build --release
    
    echo "Setting up environment..."
    if [ ! -f .env ]; then
        cp config/database.env .env
        echo "JWT_SECRET=\$(openssl rand -base64 32)" >> .env
    fi
    
    echo "Build completed successfully"
EOF

print_success "Build completed on remote server"

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

    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable hdd-tool-server
EOF

print_success "Systemd service configured"

# Start the service
print_status "Starting HDD Tool Server service..."
ssh "$SERVER_USER@$SERVER_IP" << EOF
    sudo systemctl restart hdd-tool-server
    sleep 3
    
    if sudo systemctl is-active --quiet hdd-tool-server; then
        echo "✅ Service started successfully"
        sudo systemctl status hdd-tool-server --no-pager
    else
        echo "❌ Service failed to start"
        sudo systemctl status hdd-tool-server --no-pager
        exit 1
    fi
EOF

# Test the deployment
print_status "Testing deployment..."
sleep 5

if curl -s "http://$SERVER_IP:3000/api/health" | grep -q "healthy"; then
    print_success "Deployment test passed - server is responding"
else
    print_error "Deployment test failed - server not responding"
    echo "Check server logs: ssh $SERVER_USER@$SERVER_IP 'sudo journalctl -u hdd-tool-server -f'"
    exit 1
fi

# Final status
print_success "Deployment completed successfully!"
echo ""
echo "📋 Deployment Summary:"
echo "====================="
echo "🌐 Server URL: http://$SERVER_IP:3000"
echo "👤 SSH Access: ssh $SERVER_USER@$SERVER_IP"
echo "📁 Project Dir: $REMOTE_DIR"
echo "🔧 Service: sudo systemctl status hdd-tool-server"
echo "📊 Logs: sudo journalctl -u hdd-tool-server -f"
echo ""
echo "🔐 Default Credentials:"
echo "  Admin: admin / admin123"
echo "  User: user / user123"
echo ""
echo "🎯 Next Steps:"
echo "  1. Access the web interface at http://$SERVER_IP:3000"
echo "  2. Change default passwords"
echo "  3. Configure firewall if needed"
echo "  4. Set up SSL/TLS with reverse proxy (nginx/apache)"

print_success "HDD Tool Server deployment complete!"