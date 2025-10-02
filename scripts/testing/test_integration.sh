#!/bin/bash

# Test script to verify the HDD Tool server and database connection
echo "🧪 Testing HDD Tool Server Integration..."
echo ""

# Test 1: Check if server is running locally
echo "1️⃣ Testing local server connection..."
if curl -s http://localhost:3000/api/health > /dev/null; then
    echo "✅ Local server is running"
    
    # Test health endpoint
    echo "📋 Health check response:"
    curl -s http://localhost:3000/api/health | jq '.' || curl -s http://localhost:3000/api/health
    echo ""
else
    echo "❌ Local server is not running"
    echo "   Start the server first with: sudo systemctl start hdd-tool-server"
    echo ""
fi

# Test 2: Check if remote server is accessible
echo "2️⃣ Testing remote server connection (10.0.43.207:3000)..."
if curl -s --connect-timeout 5 http://10.0.43.207:3000/api/health > /dev/null; then
    echo "✅ Remote server is accessible"
    
    # Test health endpoint
    echo "📋 Remote health check response:"
    curl -s http://10.0.43.207:3000/api/health | jq '.' || curl -s http://10.0.43.207:3000/api/health
    echo ""
else
    echo "❌ Remote server is not accessible"
    echo "   Check if server is running and firewall allows port 3000"
    echo ""
fi

# Test 3: Test login with default credentials
echo "3️⃣ Testing login functionality..."
SERVER_URL="http://localhost:3000"

# Try to login with admin/root
echo "🔐 Testing login with admin/root..."
LOGIN_RESPONSE=$(curl -s -X POST "$SERVER_URL/api/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"admin","password":"root"}')

echo "Login response:"
echo "$LOGIN_RESPONSE" | jq '.' 2>/dev/null || echo "$LOGIN_RESPONSE"
echo ""

# Test 4: Test database connection directly
echo "4️⃣ Testing direct database connection..."
if command -v psql &> /dev/null; then
    echo "🐘 Testing PostgreSQL connection..."
    if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT current_timestamp, version();' 2>/dev/null; then
        echo "✅ Database connection successful"
        
        # Check if users exist
        echo ""
        echo "👥 Checking users in database..."
        PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT username, role, is_active FROM users;' 2>/dev/null || echo "❌ Could not query users table"
        
    else
        echo "❌ Database connection failed"
        echo "   Check PostgreSQL service: sudo systemctl status postgresql"
    fi
else
    echo "⚠️  psql not available - skipping direct database test"
fi

echo ""
echo "🎯 Test Results Summary:"
echo "─────────────────────────"

# Check results
if curl -s http://localhost:3000/api/health > /dev/null; then
    echo "✅ Server: Running locally"
else
    echo "❌ Server: Not running locally"
fi

if curl -s --connect-timeout 5 http://10.0.43.207:3000/api/health > /dev/null; then
    echo "✅ Network: Remote server accessible"
else
    echo "❌ Network: Remote server not accessible"
fi

if PGPASSWORD=root psql -h localhost -U hdd_user -d hdd_tool_db -c 'SELECT 1;' &>/dev/null; then
    echo "✅ Database: Connection working"
else
    echo "❌ Database: Connection failed"
fi

echo ""
echo "🔧 Next Steps:"
echo "──────────────"
echo "1. If server is not running: sudo systemctl start hdd-tool-server"
echo "2. If database fails: sudo systemctl restart postgresql"
echo "3. Check logs: sudo journalctl -u hdd-tool-server -f"
echo "4. Open firewall: sudo ufw allow 3000"
echo ""
echo "🌐 Server URLs:"
echo "   Local:  http://localhost:3000"
echo "   Remote: http://10.0.43.207:3000"
echo "   API Health: /api/health"
echo "   API Login: /api/login"