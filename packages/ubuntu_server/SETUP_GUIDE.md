# Complete Ubuntu Server Web Setup Guide

This guide will walk you through setting up the HDD Tool web interface on your Ubuntu server step by step.

## 🚀 Quick Setup (Automated)

### Option 1: One-Command Setup
```bash
curl -sSL https://raw.githubusercontent.com/your-repo/hdd-tool/main/ubuntu_server/setup/server_setup.sh | bash
```

### Option 2: Manual Setup (Recommended)
Follow the detailed steps below for better control and understanding.

## 📋 Prerequisites

- Ubuntu Server 18.04+ (20.04 LTS recommended)
- Minimum 2GB RAM, 10GB disk space
- Root or sudo access
- Internet connection

## 🔧 Step-by-Step Manual Setup

### Step 1: Update System

```bash
# Update package lists
sudo apt update

# Upgrade existing packages
sudo apt upgrade -y

# Install essential packages
sudo apt install -y curl wget git unzip
```

### Step 2: Install Dependencies

```bash
# Install build tools
sudo apt install -y build-essential pkg-config libssl-dev

# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Install Nginx (web server/reverse proxy)
sudo apt install -y nginx

# Install Node.js (for any future frontend needs)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
```

### Step 3: Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust to PATH
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Step 4: Setup PostgreSQL Database

```bash
# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres psql << 'EOF'
CREATE DATABASE hdd_tool_db;
CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'root';
GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;
ALTER DATABASE hdd_tool_db OWNER TO hdd_user;
\q
EOF

# Test database connection
PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT version();'
```

### Step 5: Create Project Directory

```bash
# Create project directory
mkdir -p ~/hdd-tool-server
cd ~/hdd-tool-server

# Create directory structure
mkdir -p src/{api,auth,database}
mkdir -p static/{css,js}
mkdir -p templates
mkdir -p config
```

### Step 6: Copy Server Files

You have two options here:

#### Option A: Manual File Transfer (Recommended)

1. **From your Windows machine, copy the server directory:**
   ```cmd
   # Using SCP (if you have SSH access)
   scp -r E:\SIH\HDD-Tool\server\* username@your-server-ip:~/hdd-tool-server/
   
   # Or use WinSCP, FileZilla, or similar tools
   ```

2. **Or create the files manually on the server** (I'll provide the key files below)

#### Option B: Clone from Repository
```bash
# If you have the code in a git repository
git clone your-repository-url
cp -r repository-name/server/* ~/hdd-tool-server/
```

### Step 7: Create Essential Files

If you're setting up manually, create these key files:

#### Create Cargo.toml
```bash
cat > ~/hdd-tool-server/Cargo.toml << 'EOF'
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
```

#### Create Environment File
```bash
cat > ~/hdd-tool-server/.env << 'EOF'
DATABASE_URL=postgresql://hdd_user:root@localhost/hdd_tool_db
PORT=3000
HOST=0.0.0.0
JWT_SECRET=hdd-tool-server-super-secret-key-change-in-production
RUST_LOG=info
EOF
```

### Step 8: Build the Server

```bash
cd ~/hdd-tool-server

# Build the project
cargo build --release

# This will take some time on first build (downloading dependencies)
```

### Step 9: Setup Systemd Service

```bash
# Create systemd service file
sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << 'EOF'
[Unit]
Description=HDD Tool Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/$USER/hdd-tool-server
Environment=PATH=/home/$USER/.cargo/bin:$PATH
EnvironmentFile=/home/$USER/hdd-tool-server/.env
ExecStart=/home/$USER/hdd-tool-server/target/release/hdd-tool-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Replace $USER with actual username
sudo sed -i "s/\$USER/$USER/g" /etc/systemd/system/hdd-tool-server.service

# Reload systemd and enable service
sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server
```

### Step 10: Setup Nginx Reverse Proxy

```bash
# Get your server IP
SERVER_IP=$(hostname -I | awk '{print $1}')

# Create Nginx configuration
sudo tee /etc/nginx/sites-available/hdd-tool << EOF
server {
    listen 80;
    server_name $SERVER_IP localhost;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Main application
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
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Static files (if any)
    location /static/ {
        proxy_pass http://127.0.0.1:3000/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:3000/api/;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable the site
sudo ln -sf /etc/nginx/sites-available/hdd-tool /etc/nginx/sites-enabled/

# Remove default site
sudo rm -f /etc/nginx/sites-enabled/default

# Test Nginx configuration
sudo nginx -t

# Restart Nginx
sudo systemctl restart nginx
sudo systemctl enable nginx
```

### Step 11: Configure Firewall

```bash
# Install UFW if not installed
sudo apt install -y ufw

# Configure firewall rules
sudo ufw allow ssh
sudo ufw allow 80/tcp comment "HTTP"
sudo ufw allow 443/tcp comment "HTTPS"
sudo ufw allow 3000/tcp comment "HDD Tool Direct"

# Enable firewall
sudo ufw --force enable

# Check status
sudo ufw status
```

### Step 12: Start the Services

```bash
# Start the HDD Tool server
sudo systemctl start hdd-tool-server

# Check service status
sudo systemctl status hdd-tool-server

# Check logs
sudo journalctl -u hdd-tool-server -f
```

## 🌐 Access Your Web Interface

Once everything is set up:

1. **Find your server IP:**
   ```bash
   hostname -I | awk '{print $1}'
   ```

2. **Access the web interface:**
   - Main interface: `http://YOUR_SERVER_IP`
   - API health check: `http://YOUR_SERVER_IP/api/health`
   - Direct server: `http://YOUR_SERVER_IP:3000` (if firewall allows)

3. **Default login credentials:**
   - Admin: `admin` / `admin123`
   - User: `user` / `user123`

## 🔧 Verification & Testing

### Check All Services
```bash
# Check PostgreSQL
sudo systemctl status postgresql

# Check Nginx
sudo systemctl status nginx

# Check HDD Tool Server
sudo systemctl status hdd-tool-server

# Check open ports
sudo netstat -tulpn | grep -E ":80|:3000|:5432"
```

### Test Web Interface
```bash
# Test health endpoint
curl http://localhost/api/health

# Test direct server
curl http://localhost:3000/api/health

# Test from external machine
curl http://YOUR_SERVER_IP/api/health
```

## 🚨 Troubleshooting

### Common Issues

1. **Service won't start:**
   ```bash
   # Check logs
   sudo journalctl -u hdd-tool-server -f
   
   # Check if port is in use
   sudo netstat -tulpn | grep :3000
   ```

2. **Database connection issues:**
   ```bash
   # Test database manually
   PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;'
   
   # Check PostgreSQL status
   sudo systemctl status postgresql
   ```

3. **Build failures:**
   ```bash
   # Update Rust
   rustup update
   
   # Clean and rebuild
   cargo clean
   cargo build --release
   ```

4. **Nginx issues:**
   ```bash
   # Test configuration
   sudo nginx -t
   
   # Check logs
   sudo tail -f /var/log/nginx/error.log
   ```

### Log Locations
- **HDD Tool Server:** `sudo journalctl -u hdd-tool-server`
- **Nginx:** `/var/log/nginx/access.log` and `/var/log/nginx/error.log`
- **PostgreSQL:** `/var/log/postgresql/`

## 🔄 Management Commands

```bash
# Restart services
sudo systemctl restart hdd-tool-server
sudo systemctl restart nginx
sudo systemctl restart postgresql

# View logs
sudo journalctl -u hdd-tool-server -f
sudo tail -f /var/log/nginx/access.log

# Update application
cd ~/hdd-tool-server
git pull  # if using git
cargo build --release
sudo systemctl restart hdd-tool-server
```

## 🎯 Next Steps

1. **Configure your Windows client** to connect to the server
2. **Change default passwords** for security
3. **Set up SSL/HTTPS** for production use
4. **Configure automated backups** for the database
5. **Set up monitoring** and log rotation

Your HDD Tool web interface should now be accessible at `http://YOUR_SERVER_IP`! 🎉