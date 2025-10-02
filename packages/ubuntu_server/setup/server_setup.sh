#!/bin/bash

# HDD Tool Server Setup Script for Ubuntu
# Complete setup for Ubuntu server environment

set -e

echo "🚀 HDD Tool Server Setup for Ubuntu"
echo "===================================="

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root"
   exit 1
fi

# Get server IP address
SERVER_IP=$(hostname -I | awk '{print $1}')
print_status "Detected server IP: $SERVER_IP"

# Update system
print_status "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install dependencies
print_status "Installing required packages..."
sudo apt install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql \
    postgresql-contrib \
    git \
    ufw \
    nginx

# Install Rust
print_status "Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed successfully"
else
    print_warning "Rust is already installed"
fi

# Setup PostgreSQL
print_status "Setting up PostgreSQL..."
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
print_status "Creating database..."
sudo -u postgres psql << EOF
CREATE DATABASE IF NOT EXISTS hdd_tool_db;
CREATE USER IF NOT EXISTS hdd_user WITH ENCRYPTED PASSWORD 'root';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER DATABASE hdd_tool_db OWNER TO hdd_user;
\q
EOF

# Test database connection
print_status "Testing database connection..."
if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;' &>/dev/null; then
    print_success "Database connection successful"
else
    print_error "Database connection failed"
    exit 1
fi

# Create project directory
PROJECT_DIR="$HOME/hdd-tool-server"
print_status "Creating project directory: $PROJECT_DIR"
mkdir -p "$PROJECT_DIR"

# Setup firewall
print_status "Configuring firewall..."
sudo ufw allow ssh
sudo ufw allow 3000/tcp comment "HDD Tool Server"
sudo ufw allow 80/tcp comment "Nginx"
sudo ufw allow 443/tcp comment "Nginx HTTPS"
sudo ufw --force enable

# Create nginx configuration
print_status "Setting up Nginx reverse proxy..."
sudo tee /etc/nginx/sites-available/hdd-tool << EOF
server {
    listen 80;
    server_name $SERVER_IP;

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
sudo nginx -t && sudo systemctl restart nginx

print_success "Server setup completed!"
echo ""
echo "📋 Server Information:"
echo "====================="
echo "🌐 Server IP: $SERVER_IP"
echo "🔗 Web Interface: http://$SERVER_IP"
echo "🔗 Direct API: http://$SERVER_IP:3000"
echo "📁 Project Directory: $PROJECT_DIR"
echo ""
echo "📝 Next Steps:"
echo "1. Copy server source files to: $PROJECT_DIR"
echo "2. Build the project: cd $PROJECT_DIR && cargo build --release"
echo "3. Configure and start the service"
echo ""
echo "🔐 Database Credentials:"
echo "   Host: localhost"
echo "   Database: hdd_tool_db"
echo "   User: hdd_user"
echo "   Password: root"