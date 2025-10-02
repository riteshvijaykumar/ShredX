#!/bin/bash

# Quick HDD Tool Server Troubleshooting Script

echo "======================================"
echo "HDD Tool Server Troubleshooting"
echo "======================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    print_error "This script must be run as root. Use: sudo $0"
    exit 1
fi

# 1. Check service status
echo "1. Service Status:"
echo "----------------------------------------"
for service in hdd-tool-server nginx postgresql; do
    if systemctl is-active --quiet $service; then
        print_status "$service: Running"
    else
        print_error "$service: Not Running"
        echo "   To start: systemctl start $service"
    fi
done

# 2. Check if binary exists
echo ""
echo "2. Binary Check:"
echo "----------------------------------------"
if [ -f "/usr/local/bin/hdd-tool-server" ]; then
    print_status "Server binary exists"
    ls -la /usr/local/bin/hdd-tool-server
else
    print_error "Server binary not found at /usr/local/bin/hdd-tool-server"
    echo "   Need to build and install the server"
fi

# 3. Check ports
echo ""
echo "3. Port Status:"
echo "----------------------------------------"
if command -v ss &> /dev/null; then
    ss -tlnp | grep -E ":80|:3030|:5432" || print_warning "No services listening on expected ports"
elif command -v netstat &> /dev/null; then
    netstat -tlnp | grep -E ":80|:3030|:5432" || print_warning "No services listening on expected ports"
else
    print_warning "Neither ss nor netstat available. Installing net-tools..."
    apt update && apt install -y net-tools
fi

# 4. Check database
echo ""
echo "4. Database Check:"
echo "----------------------------------------"
if systemctl is-active --quiet postgresql; then
    if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw hdd_tool_db; then
        print_status "Database 'hdd_tool_db' exists"
    else
        print_error "Database 'hdd_tool_db' not found"
        echo "   Creating database..."
        sudo -u postgres createdb hdd_tool_db
        sudo -u postgres psql -c "CREATE USER hdd_user WITH ENCRYPTED PASSWORD 'root';" 2>/dev/null || true
        sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;"
    fi
else
    print_error "PostgreSQL is not running"
    echo "   To start: systemctl start postgresql"
fi

# 5. Check logs
echo ""
echo "5. Recent Service Logs:"
echo "----------------------------------------"
if systemctl list-units --type=service | grep -q hdd-tool-server; then
    echo "Last 5 log entries for hdd-tool-server:"
    journalctl -u hdd-tool-server --no-pager -n 5
else
    print_warning "hdd-tool-server service not found"
fi

# 6. Network info
echo ""
echo "6. Network Information:"
echo "----------------------------------------"
SERVER_IP=$(hostname -I | awk '{print $1}')
print_status "Server IP: $SERVER_IP"

# Test connectivity
echo "Testing endpoints..."
if curl -s --connect-timeout 5 http://localhost/health &>/dev/null; then
    print_status "Local health endpoint: OK"
else
    print_error "Local health endpoint: Failed"
fi

# 7. Quick fixes
echo ""
echo "7. Quick Fix Commands:"
echo "----------------------------------------"
echo "# Restart all services:"
echo "systemctl restart postgresql nginx hdd-tool-server"
echo ""
echo "# View live logs:"
echo "journalctl -fu hdd-tool-server"
echo ""
echo "# Rebuild and reinstall server:"
echo "cd /opt/hdd-tool-server && cargo build --release"
echo "cp target/release/hdd-tool-server /usr/local/bin/"
echo "systemctl restart hdd-tool-server"
echo ""
echo "# Reset database:"
echo "sudo -u postgres dropdb hdd_tool_db"
echo "sudo -u postgres createdb hdd_tool_db"
echo "sudo -u postgres psql -c \"GRANT ALL PRIVILEGES ON DATABASE hdd_tool_db TO hdd_user;\""

echo ""
print_status "Troubleshooting complete!"