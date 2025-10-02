# 🚀 Complete HDD Tool Setup and Testing Guide

## Overview
This guide will help you set up the complete HDD Tool system with certificate generation and server storage.

## Prerequisites
- Ubuntu Server (10.0.43.207 or your server IP)
- PostgreSQL installed and configured
- HDD Tool desktop application on Windows

## Part 1: Server Setup

### Step 1: Connect to Your Ubuntu Server
```bash
ssh your-username@10.0.43.207
```

### Step 2: Quick Server Test
```bash
# Download and run the test script
chmod +x test_server_setup.sh
./test_server_setup.sh
```

### Step 3: Full Server Setup (if test passes)
```bash
# Run the complete setup script
chmod +x server_setup.sh
sudo ./server_setup.sh
```

### Step 4: Verify Server is Running
```bash
# Check service status
sudo systemctl status hdd-tool-server

# Test the endpoints
curl http://localhost:3000/api/health

# Test login
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"root"}'
```

## Part 2: Desktop Application Configuration

### Step 1: Update config.json
Edit your `config.json` file:
```json
{
  "server_url": "http://10.0.43.207:3000",
  "enable_server_sync": true,
  "auto_upload_certificates": true,
  "local_storage_only": false,
  "connection_timeout_seconds": 30
}
```

### Step 2: Run the HDD Tool Application
```bash
# In your Windows machine
cd E:\SIH\HDD-Tool
cargo run
```

## Part 3: Complete Testing Workflow

### Step 1: Authentication
1. Launch the HDD Tool application
2. You should see the server login screen
3. Login with credentials: `admin` / `root`

### Step 2: Drive Selection and Sanitization
1. Go to the "Drives" tab
2. Select a **non-system drive** (NOT C:) for testing
   - ⚠️ **WARNING**: Never select C: drive as it will make your system unbootable
   - Use a USB drive, external drive, or secondary drive for testing
3. Choose sanitization method (NIST SP 800-88 recommended)
4. Check "Confirm to erase the data"
5. Click "🔥 Start Erase Process"

### Step 3: Monitor Progress
1. Watch the sanitization progress in the "Report" tab
2. The process will show real-time progress for each drive
3. Wait for completion message

### Step 4: Certificate Generation and Storage
When sanitization completes:
1. Certificates are automatically generated
2. Saved locally in `./certificates/` folder
3. If server sync is enabled, automatically uploaded to server
4. Go to "Certificates" tab to view all certificates

### Step 5: Verify Server Storage
Check that certificates are stored on the server:

```bash
# SSH into your server
ssh your-username@10.0.43.207

# Check database for certificates
PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db \
  -c "SELECT id, device_info, created_at FROM certificates ORDER BY created_at DESC LIMIT 5;"
```

### Step 6: Web Dashboard Access
1. Open browser and go to: `http://10.0.43.207:3000`
2. Login with same credentials
3. View all certificates and user activity

## Part 4: Integration Testing Script

Use the integration test script to verify everything works:

```bash
# On your Ubuntu server
chmod +x test_integration.sh
./test_integration.sh
```

## Expected Workflow:

```
1. User logs into desktop app ✅
2. User selects drive for sanitization ✅  
3. Sanitization process runs ✅
4. Certificate is generated automatically ✅
5. Certificate is saved locally ✅
6. Certificate is uploaded to server ✅
7. Certificate is stored in PostgreSQL database ✅
8. Certificate is viewable in web dashboard ✅
9. User can download certificate from server ✅
```

## Troubleshooting

### Server Issues
```bash
# Check server logs
sudo journalctl -u hdd-tool-server -f

# Restart server
sudo systemctl restart hdd-tool-server

# Check database
sudo systemctl status postgresql
```

### Desktop App Issues
```bash
# Check config.json file
cat config.json

# Run with debug output
RUST_LOG=debug cargo run
```

### Network Issues
```bash
# Test server connectivity
curl -v http://10.0.43.207:3000/api/health

# Check firewall
sudo ufw status

# Open port if needed
sudo ufw allow 3000
```

## Security Notes

⚠️ **Important Security Considerations:**
1. **Never test on your system drive (C:)** - it will destroy your OS
2. **Always backup important data** before testing
3. **Use test drives or USB drives** for initial testing
4. **Change default passwords** in production
5. **Use HTTPS in production** (not HTTP)

## File Locations

### On Desktop (Windows):
- Application: `E:\SIH\HDD-Tool\target\release\hdd_tool.exe`
- Config: `E:\SIH\HDD-Tool\config.json`
- Local Certificates: `E:\SIH\HDD-Tool\certificates\`
- Reports: `E:\SIH\HDD-Tool\sanitization_report_*.txt`

### On Server (Ubuntu):
- Server Binary: `/opt/hdd-tool/target/release/hdd-tool-server`
- Service: `/etc/systemd/system/hdd-tool-server.service`
- Logs: `sudo journalctl -u hdd-tool-server`
- Database: PostgreSQL `hdd_tool_db` database

## Success Indicators

✅ **You'll know it's working when:**
1. You can login to desktop app with server credentials
2. You can see your drives and select one for sanitization
3. Sanitization progress shows real-time updates
4. Certificate appears in "Certificates" tab after completion
5. Server database contains the certificate record
6. Web dashboard shows the certificate

## Next Steps After Successful Test

1. **Scale Testing**: Test with different drive types and sizes
2. **User Management**: Create additional users with different roles
3. **Compliance**: Test with different sanitization standards
4. **Reporting**: Export and audit certificate reports
5. **Production Setup**: Implement HTTPS and production security