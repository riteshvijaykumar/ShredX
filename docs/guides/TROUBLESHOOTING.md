# 🔧 HDD Tool Server Setup Troubleshooting Guide

## Problem: Server setup script lines not executing

### Quick Diagnosis

Run these commands in your SSH session to check what might be wrong:

```bash
# 1. Check if you're in the right location
pwd
ls -la

# 2. Check if previous steps completed
echo "Testing database connection..."
PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT version();'

# 3. Check Rust installation
rustc --version
which cargo

# 4. Check if /opt/hdd-tool exists and has right permissions
ls -la /opt/hdd-tool
whoami
```

### Common Issues & Solutions

#### Issue 1: Script stopped at previous step
**Symptoms**: Lines not executing, script seems to hang
**Solution**: 
```bash
# Check if any processes are stuck
ps aux | grep -E "(psql|cargo|rustc)"

# Kill any stuck processes
sudo pkill -f psql
sudo pkill -f cargo

# Restart from the failed step
cd /opt/hdd-tool
```

#### Issue 2: Permission problems
**Symptoms**: "Permission denied" errors
**Solution**:
```bash
# Fix ownership
sudo chown -R $USER:$USER /opt/hdd-tool
cd /opt/hdd-tool

# Make sure you have write permissions
chmod 755 /opt/hdd-tool
```

#### Issue 3: Rust environment not loaded
**Symptoms**: "cargo: command not found"
**Solution**:
```bash
# Reload Rust environment
source ~/.cargo/env

# Verify it's working
cargo --version
```

#### Issue 4: Database connection issues
**Symptoms**: Database connection failures
**Solution**:
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Restart if needed
sudo systemctl restart postgresql

# Test connection manually
PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c '\l'
```

### Step-by-Step Manual Execution

If the automated script isn't working, run these commands one by one:

```bash
# Go to the application directory
cd /opt/hdd-tool

# Create the Rust project
cat > Cargo.toml << 'EOF'
[package]
name = "hdd_tool_server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hdd-tool-server"
path = "src/server/main.rs"

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
EOF

echo "✅ Cargo.toml created"

# Create the server directory
mkdir -p src/server

# Create the main server file (this is a long command, copy all of it)
cat > src/server/main.rs << 'EOF'
use std::sync::Arc;
use warp::Filter;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use sha2::{Sha256, Digest};
use uuid::Uuid;

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
struct RegisterRequest {
    username: String,
    email: String, 
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

            let _ = sqlx::query("UPDATE users SET last_login = NOW() WHERE id = $1")
                .bind(user.id)
                .execute(pool.as_ref())
                .await;

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
            eprintln!("Database error: {}", e);
            let response: ApiResponse<()> = ApiResponse {
                success: false,
                data: None,
                message: "Server error".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn register_handler(
    body: RegisterRequest,
    pool: Arc<PgPool>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let password_hash = hash_password(&body.password);
    
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, role) 
         VALUES ($1, $2, $3, 'user') RETURNING id"
    )
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .fetch_one(pool.as_ref())
    .await;

    match result {
        Ok(_) => {
            let response: ApiResponse<()> = ApiResponse {
                success: true,
                data: None,
                message: "User registered successfully".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            eprintln!("Registration error: {}", e);
            let response: ApiResponse<()> = ApiResponse {
                success: false,
                data: None,
                message: "Registration failed - username or email may already exist".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
    }
}

async fn health_handler() -> Result<impl warp::Reply, warp::Rejection> {
    let response = ApiResponse {
        success: true,
        data: Some("HDD Tool Server is running"),
        message: "Server healthy".to_string(),
    };
    Ok(warp::reply::json(&response))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    let database_url = "postgresql://hdd_user:root@localhost/hdd_tool_db";
    let pool = Arc::new(PgPool::connect(database_url).await?);

    println!("🚀 HDD Tool Server starting...");
    println!("📊 Database: Connected to PostgreSQL");
    println!("🌐 Server URL: http://0.0.0.0:3000");
    println!("👤 Default users:");
    println!("   - admin / root (admin role)");
    println!("   - user / root (user role)");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

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

    let register = warp::path("api")
        .and(warp::path("register"))
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || pool.clone()))
        .and_then(register_handler);

    let routes = health
        .or(login)
        .or(register)
        .with(cors);

    println!("✅ Server started successfully!");
    println!("📋 Available endpoints:");
    println!("   GET  /api/health   - Health check");
    println!("   POST /api/login    - User login");
    println!("   POST /api/register - User registration");

    warp::serve(routes)
        .run(([0, 0, 0, 0], 3000))
        .await;

    Ok(())
}
EOF

echo "✅ Server code created"

# Load Rust environment
echo "🦀 Loading Rust environment..."
source ~/.cargo/env

# Build the server
echo "🔨 Building server (this may take a few minutes)..."
cargo build --release --bin hdd-tool-server

# Check if build succeeded
if [ -f "target/release/hdd-tool-server" ]; then
    echo "✅ Build successful!"
    ls -la target/release/hdd-tool-server
else
    echo "❌ Build failed!"
    echo "Check the error messages above"
    exit 1
fi

# Create systemd service
echo "⚙️ Creating systemd service..."
sudo tee /etc/systemd/system/hdd-tool-server.service > /dev/null << EOF
[Unit]
Description=HDD Tool Authentication Server
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$USER
WorkingDirectory=/opt/hdd-tool
ExecStart=/opt/hdd-tool/target/release/hdd-tool-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start the service
sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server
sudo systemctl start hdd-tool-server

# Wait a moment for startup
sleep 3

# Check if it's running
echo "🔍 Checking service status..."
sudo systemctl status hdd-tool-server --no-pager

# Test the endpoints
echo "🧪 Testing endpoints..."
curl -s http://localhost:3000/api/health && echo ""

echo "🎉 Server setup complete!"
```

### Final Verification

Once everything is running, test with these commands:

```bash
# Health check
curl http://localhost:3000/api/health

# Login test
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"root"}'

# Check service status
sudo systemctl status hdd-tool-server

# View logs
sudo journalctl -u hdd-tool-server -f
```

### If You Still Have Issues

1. **Run the test script first**: Use `test_server_setup.sh` to identify specific problems
2. **Check logs**: `sudo journalctl -u hdd-tool-server -f`
3. **Verify network**: Make sure port 3000 is accessible
4. **Check firewall**: `sudo ufw status`

### Getting Help

If you're still having issues, provide these details:
- Which step is failing?
- What error messages do you see?
- Output of `sudo systemctl status hdd-tool-server`
- Output of `sudo journalctl -u hdd-tool-server -n 50`