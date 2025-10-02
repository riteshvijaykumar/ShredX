# Quick Manual Fix for 502 Bad Gateway Error

## The Problem
502 Bad Gateway means Nginx can't connect to your backend server on port 3030.

## Quick Diagnosis Commands:
```bash
# Check if backend server is running
sudo systemctl status hdd-tool-server

# Check if anything is listening on port 3030
sudo ss -tlnp | grep :3030

# Check recent logs
sudo journalctl -u hdd-tool-server -n 20
```

## Quick Fix Commands (run as root):

### 1. Install missing tools if needed:
```bash
sudo apt update
sudo apt install -y net-tools curl
```

### 2. Check if server binary exists:
```bash
ls -la /usr/local/bin/hdd-tool-server
```

### 3. If binary doesn't exist, build it:
```bash
cd /home/*/hdd-tool-server  # or wherever your project is
cargo build --release
sudo cp target/release/hdd-tool-server /usr/local/bin/
sudo chmod +x /usr/local/bin/hdd-tool-server
```

### 4. Create/fix the systemd service:
```bash
sudo tee /etc/systemd/system/hdd-tool-server.service << 'EOF'
[Unit]
Description=HDD Tool Server
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/hdd-tool-server
ExecStart=/usr/local/bin/hdd-tool-server
Environment=SERVER_PORT=3030
Environment=RUST_LOG=info
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
```

### 5. Restart services:
```bash
sudo systemctl daemon-reload
sudo systemctl enable hdd-tool-server
sudo systemctl restart hdd-tool-server
sudo systemctl restart nginx
```

### 6. Check if it's working:
```bash
# Wait 5 seconds then test
sleep 5

# Test backend directly
curl http://localhost:3030/health

# Test through Nginx
curl http://localhost/health

# Check service status
sudo systemctl status hdd-tool-server
sudo systemctl status nginx
```

### 7. If still not working, try running manually:
```bash
# Stop the service
sudo systemctl stop hdd-tool-server

# Run manually to see errors
sudo /usr/local/bin/hdd-tool-server

# If it works manually, then restart service:
# Press Ctrl+C to stop manual run, then:
sudo systemctl start hdd-tool-server
```

## Common Issues:

1. **Binary doesn't exist**: Build with `cargo build --release`
2. **Permission issues**: Run service as root temporarily
3. **Port already in use**: Check with `sudo ss -tlnp | grep :3030`
4. **Rust not installed**: Install with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Test URLs after fix:
- Health check: http://YOUR_SERVER_IP/health  
- Direct backend: http://YOUR_SERVER_IP:3030/health