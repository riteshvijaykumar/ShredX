# HDD Tool Ubuntu Server Setup

This directory contains everything needed to set up and deploy the HDD Tool server on Ubuntu and configure your Windows client to connect to it.

## 📁 Directory Structure

```
ubuntu_server/
├── setup/
│   └── server_setup.sh          # Complete Ubuntu server setup script
├── deployment/
│   └── deploy.sh                # Deploy from Windows to Ubuntu
├── config/
│   └── server.env               # Server environment configuration
├── client_config/
│   ├── config_template.json     # Client configuration template
│   └── configure_client.bat     # Windows client configuration script
└── README.md                    # This file
```

## 🚀 Quick Start Guide

### Step 1: Setup Ubuntu Server

1. **SSH into your Ubuntu server:**
   ```bash
   ssh your-username@YOUR_SERVER_IP
   ```

2. **Download and run the setup script:**
   ```bash
   curl -sSL https://raw.githubusercontent.com/your-repo/hdd-tool/main/ubuntu_server/setup/server_setup.sh | bash
   ```

   Or manually copy and run `setup/server_setup.sh`

### Step 2: Deploy Server Code

From your Windows machine:

1. **Make sure you have SSH access to your Ubuntu server**
2. **Run the deployment script:**
   ```bash
   ./ubuntu_server/deployment/deploy.sh YOUR_SERVER_IP ubuntu
   ```

   Replace `YOUR_SERVER_IP` with your actual server IP address.

### Step 3: Configure Windows Client

1. **Navigate to your HDD Tool directory**
2. **Run the client configuration script:**
   ```cmd
   ubuntu_server\client_config\configure_client.bat
   ```
3. **Enter your server IP when prompted**
4. **Start your HDD Tool desktop application**

## 🔧 Manual Setup Instructions

### Ubuntu Server Setup

1. **Install dependencies:**
   ```bash
   sudo apt update
   sudo apt install -y curl build-essential pkg-config libssl-dev postgresql postgresql-contrib git nginx
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Setup PostgreSQL:**
   ```bash
   sudo -u postgres createdb hdd_tool_db
   sudo -u postgres createuser hdd_user --pwprompt  # password: root
   sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;"
   ```

4. **Configure firewall:**
   ```bash
   sudo ufw allow ssh
   sudo ufw allow 3000/tcp
   sudo ufw allow 80/tcp
   sudo ufw enable
   ```

### Server Deployment

1. **Copy server files to Ubuntu:**
   ```bash
   rsync -avz server/ username@server-ip:~/hdd-tool-server/
   ```

2. **Build on server:**
   ```bash
   ssh username@server-ip
   cd ~/hdd-tool-server
   cargo build --release
   ```

3. **Setup systemd service:**
   ```bash
   sudo cp hdd-tool-server.service /etc/systemd/system/
   sudo systemctl enable hdd-tool-server
   sudo systemctl start hdd-tool-server
   ```

## 🌐 Network Configuration

### Server Endpoints

- **Web Interface:** `http://YOUR_SERVER_IP`
- **API Endpoint:** `http://YOUR_SERVER_IP/api`
- **Health Check:** `http://YOUR_SERVER_IP/api/health`

### Firewall Ports

- **22** - SSH access
- **80** - HTTP (Nginx reverse proxy)
- **3000** - Direct server access (optional)
- **443** - HTTPS (if SSL configured)

## 🔐 Authentication

### Default Web Credentials

- **Admin:** `admin` / `admin123`
- **User:** `user` / `user123`

### Database Credentials

- **Host:** `localhost`
- **Database:** `hdd_tool_db`
- **User:** `hdd_user`
- **Password:** `root`

## 🔄 Client-Server Connection

### Windows Client Configuration

Your Windows HDD Tool app needs these settings in `config.json`:

```json
{
  "server_config": {
    "is_server_enabled": true,
    "server_url": "http://YOUR_SERVER_IP",
    "api_base_url": "http://YOUR_SERVER_IP/api"
  }
}
```

### Connection Flow

1. **Windows Client** → connects to → **Ubuntu Server**
2. **Authentication** via server credentials
3. **Drive operations** executed on server
4. **Results** synchronized back to client

## 📊 Monitoring & Management

### Check Server Status

```bash
# Service status
sudo systemctl status hdd-tool-server

# View logs
sudo journalctl -u hdd-tool-server -f

# Restart service
sudo systemctl restart hdd-tool-server
```

### Test Connection

```bash
# From any machine
curl http://YOUR_SERVER_IP/api/health

# Should return: {"status":"healthy",...}
```

## 🚨 Troubleshooting

### Common Issues

1. **Connection Refused**
   - Check firewall: `sudo ufw status`
   - Verify service: `sudo systemctl status hdd-tool-server`
   - Check port: `netstat -tulpn | grep :3000`

2. **Database Connection Failed**
   - Test connection: `PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;'`
   - Check PostgreSQL: `sudo systemctl status postgresql`

3. **Build Failures**
   - Check Rust: `rustc --version`
   - Update dependencies: `cargo update`
   - Check disk space: `df -h`

### Log Locations

- **Application logs:** `sudo journalctl -u hdd-tool-server`
- **Nginx logs:** `/var/log/nginx/access.log`
- **PostgreSQL logs:** `/var/log/postgresql/`

## 🔒 Security Considerations

1. **Change default passwords** immediately after setup
2. **Use HTTPS** in production (configure SSL certificate)
3. **Restrict firewall** to only necessary IPs
4. **Regular updates** of system packages
5. **Backup database** regularly

## 📝 Environment Variables

Copy `config/server.env` to your server as `.env`:

```bash
DATABASE_URL=postgresql://hdd_user:root@localhost/hdd_tool_db
PORT=3000
JWT_SECRET=your-secret-key
RUST_LOG=info
```

## 🎯 Production Deployment

For production use:

1. **Setup SSL/TLS certificate**
2. **Configure domain name**
3. **Enable HTTPS redirect**
4. **Setup database backups**
5. **Configure log rotation**
6. **Setup monitoring**

## 📞 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Verify all prerequisites are installed
3. Check firewall and network settings
4. Review server logs for error messages

## 🔄 Updates

To update the server:

1. Deploy new code: `./deployment/deploy.sh YOUR_SERVER_IP`
2. Restart service: `sudo systemctl restart hdd-tool-server`
3. Verify health: `curl http://YOUR_SERVER_IP/api/health`