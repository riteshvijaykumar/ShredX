# HDD Tool Server

A web-based interface for the HDD Tool that runs on Ubuntu Server, providing remote access to hard drive sanitization functionality through a client-server architecture.

## 🏗️ Architecture

```
┌─────────────────┐    HTTP/HTTPS    ┌─────────────────┐
│   Client        │ ◄─────────────► │   Ubuntu Server │
│   (Windows PC)  │                  │   (HDD Tool)    │
│   Web Browser   │                  │   Web Interface │
└─────────────────┘                  └─────────────────┘
```

## 🚀 Features

- **Web-based Interface**: Access from any device with a web browser
- **User Authentication**: Secure login with role-based access (Admin/User)
- **Drive Management**: Scan and list available hard drives
- **Sanitization Control**: Configure and start drive sanitization jobs
- **Real-time Progress**: Monitor sanitization progress in real-time
- **Audit Logging**: Complete audit trail of all operations
- **RESTful API**: Full API for integration with other tools

## 📋 Prerequisites

- Ubuntu Server 18.04+ (20.04 LTS recommended)
- Minimum 2GB RAM
- Network connectivity between client and server
- PostgreSQL database
- Rust toolchain

## 🛠️ Installation

### Quick Setup (Ubuntu Server)

1. **Download and run the setup script:**
   ```bash
   curl -sSL https://raw.githubusercontent.com/your-repo/hdd-tool/main/server/scripts/server_setup.sh | bash
   ```

2. **Copy source files to the project directory:**
   ```bash
   # Copy the entire server directory to ~/hdd-tool-server/
   ```

3. **Build and start the server:**
   ```bash
   cd ~/hdd-tool-server
   cargo build --release
   sudo systemctl start hdd-tool-server
   sudo systemctl enable hdd-tool-server
   ```

### Manual Setup

1. **Install dependencies:**
   ```bash
   sudo apt update
   sudo apt install -y curl build-essential pkg-config libssl-dev postgresql postgresql-contrib
   ```

2. **Install Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

3. **Setup PostgreSQL:**
   ```bash
   sudo -u postgres createdb hdd_tool_db
   sudo -u postgres createuser hdd_user --pwprompt
   sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;"
   ```

4. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd hdd-tool-server
   cargo build --release
   ```

5. **Configure environment:**
   ```bash
   cp config/database.env .env
   # Edit .env with your database credentials
   ```

6. **Run the server:**
   ```bash
   ./target/release/hdd-tool-server
   ```

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
DATABASE_URL=postgresql://hdd_user:password@localhost/hdd_tool_db
PORT=3000
JWT_SECRET=your-secret-key
RUST_LOG=info
```

### Database Schema

The server automatically creates the required database tables on first run:
- `users` - User authentication and authorization
- `sanitization_jobs` - Track sanitization operations
- `audit_logs` - Audit trail of all operations

## 🌐 Usage

### Web Interface

1. **Access the web interface:**
   ```
   http://your-server-ip:3000
   ```

2. **Default login credentials:**
   - Admin: `admin` / `admin123`
   - User: `user` / `user123`

3. **Main features:**
   - Dashboard with system overview
   - Drive scan and selection
   - Sanitization configuration
   - Job monitoring and progress tracking

### API Endpoints

The server provides a RESTful API:

#### Authentication
- `POST /api/login` - User login
- `POST /api/logout` - User logout

#### Drive Management
- `GET /api/drives` - List available drives
- `POST /api/drives/scan` - Scan for new drives

#### Sanitization
- `POST /api/sanitization/start` - Start sanitization job
- `GET /api/sanitization/status/{job_id}` - Get job status
- `GET /api/sanitization/jobs` - List all jobs

#### Health Check
- `GET /api/health` - Server health status

### Example API Usage

```bash
# Login
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# List drives (with auth token)
curl -X GET http://localhost:3000/api/drives \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Start sanitization
curl -X POST http://localhost:3000/api/sanitization/start \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "drive_ids": ["sda"],
    "method": "zeros",
    "passes": 1,
    "verify": true
  }'
```

## 🔐 Security

- **Authentication**: JWT-based authentication with configurable expiration
- **Authorization**: Role-based access control (Admin/User roles)
- **HTTPS**: Supports TLS/SSL encryption (configure reverse proxy)
- **Audit Logging**: All operations are logged with user identification
- **Input Validation**: All API inputs are validated and sanitized

## 📊 Monitoring

### System Service

```bash
# Check service status
sudo systemctl status hdd-tool-server

# View logs
sudo journalctl -u hdd-tool-server -f

# Restart service
sudo systemctl restart hdd-tool-server
```

### Database Monitoring

```bash
# Connect to database
PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db

# Check active jobs
SELECT * FROM sanitization_jobs WHERE status = 'running';

# View audit logs
SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 10;
```

## 🚨 Troubleshooting

### Common Issues

1. **Database Connection Failed**
   ```bash
   # Check PostgreSQL status
   sudo systemctl status postgresql
   
   # Test connection
   PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;'
   ```

2. **Port Already in Use**
   ```bash
   # Check what's using port 3000
   sudo netstat -tulpn | grep :3000
   
   # Change port in .env file
   PORT=3001
   ```

3. **Permission Denied for Drive Access**
   ```bash
   # Add user to disk group
   sudo usermod -a -G disk $USER
   
   # Restart service
   sudo systemctl restart hdd-tool-server
   ```

### Log Analysis

```bash
# View server logs
sudo journalctl -u hdd-tool-server --since "1 hour ago"

# Enable debug logging
echo "RUST_LOG=debug" >> .env
sudo systemctl restart hdd-tool-server
```

## 🔧 Development

### Building from Source

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run with auto-reload
cargo install cargo-watch
cargo watch -x run
```

### Database Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli

# Create migration
sqlx migrate add create_users_table

# Run migrations
sqlx migrate run
```

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📞 Support

For support and questions:
- Create an issue on GitHub
- Check the troubleshooting section
- Review the API documentation