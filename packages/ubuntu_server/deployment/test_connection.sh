#!/bin/bash

# HDD Tool Server Connection Test
# Test connectivity and functionality between client and server

set -e

SERVER_IP="${1:-}"
if [ -z "$SERVER_IP" ]; then
    echo "Usage: $0 <server_ip>"
    echo "Example: $0 192.168.1.100"
    exit 1
fi

echo "🧪 Testing HDD Tool Server Connection"
echo "===================================="
echo "🎯 Target Server: $SERVER_IP"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_test() { echo -e "${BLUE}[TEST]${NC} $1"; }
print_pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
print_fail() { echo -e "${RED}[FAIL]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    print_test "$test_name"
    
    if eval "$test_command" &>/dev/null; then
        print_pass "$test_name"
        ((TESTS_PASSED++))
        return 0
    else
        print_fail "$test_name"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test 1: Basic connectivity
print_test "1. Basic network connectivity"
if ping -c 1 -W 3 "$SERVER_IP" &>/dev/null; then
    print_pass "Server is reachable"
    ((TESTS_PASSED++))
else
    print_fail "Cannot reach server"
    ((TESTS_FAILED++))
fi

# Test 2: HTTP port accessibility
print_test "2. HTTP port (80) accessibility"
if timeout 5 bash -c "echo >/dev/tcp/$SERVER_IP/80" 2>/dev/null; then
    print_pass "Port 80 is open"
    ((TESTS_PASSED++))
else
    print_fail "Port 80 is not accessible"
    ((TESTS_FAILED++))
fi

# Test 3: Direct server port accessibility
print_test "3. Server port (3000) accessibility"
if timeout 5 bash -c "echo >/dev/tcp/$SERVER_IP/3000" 2>/dev/null; then
    print_pass "Port 3000 is open"
    ((TESTS_PASSED++))
else
    print_warn "Port 3000 is not accessible (may be behind proxy)"
    ((TESTS_FAILED++))
fi

# Test 4: Web interface availability
print_test "4. Web interface availability"
if curl -s -o /dev/null -w "%{http_code}" "http://$SERVER_IP" | grep -q "200\|301\|302"; then
    print_pass "Web interface is accessible"
    ((TESTS_PASSED++))
else
    print_fail "Web interface is not accessible"
    ((TESTS_FAILED++))
fi

# Test 5: API health endpoint
print_test "5. API health endpoint"
HEALTH_RESPONSE=$(curl -s "http://$SERVER_IP/api/health" 2>/dev/null || echo "")
if echo "$HEALTH_RESPONSE" | grep -q "healthy\|status"; then
    print_pass "API health endpoint is responding"
    echo "     Response: $HEALTH_RESPONSE"
    ((TESTS_PASSED++))
else
    print_fail "API health endpoint is not responding"
    ((TESTS_FAILED++))
fi

# Test 6: API login endpoint
print_test "6. API login endpoint accessibility"
LOGIN_RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test"}' \
    "http://$SERVER_IP/api/login" 2>/dev/null || echo "000")

if [ "$LOGIN_RESPONSE" != "000" ]; then
    print_pass "API login endpoint is accessible (HTTP $LOGIN_RESPONSE)"
    ((TESTS_PASSED++))
else
    print_fail "API login endpoint is not accessible"
    ((TESTS_FAILED++))
fi

# Test 7: Database connectivity (if we can SSH)
print_test "7. SSH connectivity (optional)"
if ssh -o ConnectTimeout=5 -o BatchMode=yes "$SERVER_IP" "echo 'SSH OK'" 2>/dev/null; then
    print_pass "SSH is accessible"
    
    # Test database if SSH works
    print_test "8. Database connectivity (via SSH)"
    if ssh "$SERVER_IP" "PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;'" &>/dev/null; then
        print_pass "Database is accessible"
        ((TESTS_PASSED++))
    else
        print_fail "Database is not accessible"
        ((TESTS_FAILED++))
    fi
    ((TESTS_PASSED++))
else
    print_warn "SSH is not accessible (not required for web access)"
fi

# Summary
echo ""
echo "📊 Test Results Summary"
echo "======================"
echo "✅ Tests Passed: $TESTS_PASSED"
echo "❌ Tests Failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    print_pass "🎉 All critical tests passed! Server is ready for client connection."
    echo ""
    echo "🔗 Connection Information:"
    echo "   Web Interface: http://$SERVER_IP"
    echo "   API Endpoint: http://$SERVER_IP/api"
    echo ""
    echo "📱 Client Configuration:"
    echo "   Set server_url to: http://$SERVER_IP"
    echo "   Set api_base_url to: http://$SERVER_IP/api"
    echo ""
    echo "🔐 Default Credentials:"
    echo "   Admin: admin / admin123"
    echo "   User: user / user123"
elif [ $TESTS_FAILED -le 2 ]; then
    print_warn "⚠️  Some tests failed, but basic functionality should work."
    echo "   Check failed tests and server logs for issues."
else
    print_fail "❌ Multiple critical tests failed. Please check server setup."
    echo ""
    echo "🔧 Troubleshooting Steps:"
    echo "1. Verify server is running: ssh $SERVER_IP 'sudo systemctl status hdd-tool-server'"
    echo "2. Check firewall: ssh $SERVER_IP 'sudo ufw status'"
    echo "3. View server logs: ssh $SERVER_IP 'sudo journalctl -u hdd-tool-server -f'"
    echo "4. Test database: ssh $SERVER_IP 'sudo systemctl status postgresql'"
fi

exit $TESTS_FAILED