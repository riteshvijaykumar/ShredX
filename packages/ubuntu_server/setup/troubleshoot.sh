#!/bin/bash

# HDD Tool Server Troubleshooting Script
# Diagnoses common issues with the web server setup

echo "🔍 HDD Tool Server Diagnostics"
echo "==============================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_check() { echo -e "${BLUE}[CHECK]${NC} $1"; }
print_ok() { echo -e "${GREEN}[OK]${NC} $1"; }
print_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

SERVER_IP=$(hostname -I | awk '{print $1}')
USERNAME=$(whoami)
PROJECT_DIR="/home/$USERNAME/hdd-tool-server"

echo "Server IP: $SERVER_IP"
echo "User: $USERNAME"
echo "Project: $PROJECT_DIR"
echo ""

# Check 1: System services
print_check "Checking system services..."

if systemctl is-active --quiet postgresql; then
    print_ok "PostgreSQL is running"
else
    print_fail "PostgreSQL is not running"
    echo "  Fix: sudo systemctl start postgresql"
fi

if systemctl is-active --quiet nginx; then
    print_ok "Nginx is running"
else
    print_fail "Nginx is not running"
    echo "  Fix: sudo systemctl start nginx"
fi

if systemctl is-active --quiet hdd-tool-server; then
    print_ok "HDD Tool Server is running"
else
    print_fail "HDD Tool Server is not running"
    echo "  Fix: sudo systemctl start hdd-tool-server"
    echo "  Logs: sudo journalctl -u hdd-tool-server -f"
fi

echo ""

# Check 2: Network ports
print_check "Checking network ports..."

if netstat -tulpn 2>/dev/null | grep -q ":80 "; then
    print_ok "Port 80 (HTTP) is open"
else
    print_fail "Port 80 (HTTP) is not open"
fi

if netstat -tulpn 2>/dev/null | grep -q ":3000 "; then
    print_ok "Port 3000 (Server) is open"
else
    print_fail "Port 3000 (Server) is not open"
fi

if netstat -tulpn 2>/dev/null | grep -q ":5432 "; then
    print_ok "Port 5432 (PostgreSQL) is open"
else
    print_fail "Port 5432 (PostgreSQL) is not open"
fi

echo ""

# Check 3: Database connectivity
print_check "Checking database connectivity..."

if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;' &>/dev/null; then
    print_ok "Database connection successful"
else
    print_fail "Database connection failed"
    echo "  Fix: Check PostgreSQL service and credentials"
fi

echo ""

# Check 4: Project files
print_check "Checking project files..."

if [ -d "$PROJECT_DIR" ]; then
    print_ok "Project directory exists"
else
    print_fail "Project directory missing: $PROJECT_DIR"
fi

if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
    print_ok "Cargo.toml exists"
else
    print_fail "Cargo.toml missing"
fi

if [ -f "$PROJECT_DIR/.env" ]; then
    print_ok "Environment file exists"
else
    print_fail "Environment file missing"
fi

if [ -f "$PROJECT_DIR/target/release/hdd-tool-server" ]; then
    print_ok "Server binary exists"
else
    print_fail "Server binary missing - need to build"
    echo "  Fix: cd $PROJECT_DIR && cargo build --release"
fi

echo ""

# Check 5: Web connectivity
print_check "Checking web connectivity..."

if curl -s -o /dev/null -w "%{http_code}" "http://localhost/" 2>/dev/null | grep -q "200"; then
    print_ok "Web interface accessible locally"
else
    print_fail "Web interface not accessible locally"
fi

if curl -s "http://localhost/api/health" 2>/dev/null | grep -q "healthy"; then
    print_ok "API health endpoint working"
else
    print_fail "API health endpoint not working"
fi

echo ""

# Check 6: Firewall
print_check "Checking firewall..."

if command -v ufw >/dev/null && ufw status 2>/dev/null | grep -q "Status: active"; then
    if ufw status | grep -q "80/tcp"; then
        print_ok "Firewall allows HTTP (port 80)"
    else
        print_warn "Firewall may be blocking HTTP"
        echo "  Fix: sudo ufw allow 80/tcp"
    fi
else
    print_warn "UFW firewall not active or not installed"
fi

echo ""

# Check 7: Logs analysis
print_check "Recent error logs..."

if journalctl -u hdd-tool-server --since "10 minutes ago" -q | grep -i error | head -3; then
    print_warn "Found recent errors in server logs"
    echo "  View full logs: sudo journalctl -u hdd-tool-server -f"
else
    print_ok "No recent errors in server logs"
fi

echo ""

# Quick fixes section
echo "🔧 Quick Fixes:"
echo "==============="
echo ""
echo "1. Restart all services:"
echo "   sudo systemctl restart postgresql nginx hdd-tool-server"
echo ""
echo "2. Check detailed logs:"
echo "   sudo journalctl -u hdd-tool-server -f"
echo "   sudo tail -f /var/log/nginx/error.log"
echo ""
echo "3. Rebuild server (if needed):"
echo "   cd $PROJECT_DIR"
echo "   cargo build --release"
echo "   sudo systemctl restart hdd-tool-server"
echo ""
echo "4. Test connectivity:"
echo "   curl http://localhost/api/health"
echo "   curl http://$SERVER_IP/api/health"
echo ""
echo "5. View server status:"
echo "   sudo systemctl status hdd-tool-server nginx postgresql"
echo ""

# Summary
echo "📊 Diagnostic Summary:"
echo "====================="
echo "If you see any [FAIL] items above, use the suggested fixes."
echo "For detailed help, check the logs or run the setup script again."
echo ""
echo "🌐 Your server should be accessible at: http://$SERVER_IP"