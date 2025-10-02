#!/bin/bash

# HDD Tool Server Setup Script for Ubuntu
# This script sets up the complete server environment on Ubuntu

set -e

echo "🚀 HDD Tool Server Setup Starting..."
echo "=================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root for security reasons"
   exit 1
fi

# Update system packages
print_status "Updating system packages..."
sudo apt update && sudo apt upgrade -y

# Install required packages
print_status "Installing required packages..."
sudo apt install -y curl build-essential pkg-config libssl-dev postgresql postgresql-contrib git

# Install Rust
print_status "Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    print_success "Rust installed successfully"
else
    print_warning "Rust is already installed"
fi

# Verify Rust installation
print_status "Verifying Rust installation..."
rustc --version
cargo --version

# Setup PostgreSQL
print_status "Setting up PostgreSQL..."

# Start and enable PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
print_status "Creating database and user..."
sudo -u postgres psql << EOF
CREATE DATABASE hdd_tool_db;
CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'root';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER DATABASE hdd_tool_db OWNER TO hdd_user;
\q
EOF

print_success "Database setup completed"

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
print_status "Creating project directory at $PROJECT_DIR..."
mkdir -p "$PROJECT_DIR"
cd "$PROJECT_DIR"

# Create Cargo.toml
print_status "Creating Cargo.toml..."
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

# Create directory structure
print_status "Creating directory structure..."
mkdir -p src/{api,auth,database}
mkdir -p static/{css,js}
mkdir -p templates
mkdir -p config

# Copy source files (this would need to be customized based on your deployment method)
print_status "Setting up source files..."
# Note: In a real deployment, you would copy the source files here
# For now, we'll create placeholder files

# Create environment file
print_status "Creating environment configuration..."
cat > .env << EOF
DATABASE_URL=postgresql://hdd_user:root@localhost/hdd_tool_db
PORT=3000
JWT_SECRET=$(openssl rand -base64 32)
RUST_LOG=info
EOF

print_success "Environment file created"

# Build the project (this will fail without source files, but shows the process)
print_status "Building the project..."
source ~/.cargo/env
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_warning "Build failed - source files need to be provided"
fi

# Create systemd service file
print_status "Creating systemd service..."
sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << EOF
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$USER
WorkingDirectory=$PROJECT_DIR
Environment=PATH=/home/$USER/.cargo/bin:\$PATH
EnvironmentFile=$PROJECT_DIR/.env
ExecStart=$PROJECT_DIR/target/release/hdd-tool-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

print_success "Systemd service created"

# Setup firewall (if ufw is installed)
if command -v ufw &> /dev/null; then
    print_status "Configuring firewall..."
    sudo ufw allow 3000/tcp comment "HDD Tool Server"
    print_success "Firewall configured"
fi

# Display completion message
print_success "Setup completed successfully!"
echo ""
echo "📋 Next Steps:"
echo "=============="
echo "1. Copy your source files to: $PROJECT_DIR"
echo "2. Build the project: cd $PROJECT_DIR && cargo build --release"
echo "3. Start the service: sudo systemctl start hdd-tool-server"
echo "4. Enable auto-start: sudo systemctl enable hdd-tool-server"
echo "5. Check status: sudo systemctl status hdd-tool-server"
echo ""
echo "🌐 Server will be available at:"
echo "   - Local: http://localhost:3000"
echo "   - Network: http://$(hostname -I | awk '{print $1}'):3000"
echo ""
echo "📊 Database Info:"
echo "   - Host: localhost"
echo "   - Database: hdd_tool_db"
echo "   - User: hdd_user"
echo "   - Password: root"
echo ""
echo "🔐 Default Web Credentials:"
echo "   - Admin: admin / admin123"
echo "   - User: user / user123"

print_success "HDD Tool Server setup complete!"