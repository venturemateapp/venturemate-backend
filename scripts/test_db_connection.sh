#!/bin/bash

# Supabase Database Connection Test Script

echo "========================================"
echo "Supabase Connection Diagnostics"
echo "========================================"
echo ""

# Load .env if it exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Parse the database URL
if [ -z "$SUPABASE_DB_URL" ]; then
    echo "❌ SUPABASE_DB_URL is not set!"
    echo "   Please set it in your .env file"
    exit 1
fi

echo "✅ SUPABASE_DB_URL is set"
echo ""

# Extract hostname from URL
DB_HOST=$(echo "$SUPABASE_DB_URL" | sed -n 's/.*@\([^:]*\).*/\1/p')
echo "Database Host: $DB_HOST"
echo ""

# Test DNS resolution
echo "Testing DNS resolution..."
if ping -c 1 -W 5 "$DB_HOST" > /dev/null 2>&1; then
    echo "✅ Hostname resolves successfully"
else
    echo "❌ Cannot resolve hostname: $DB_HOST"
    echo ""
    echo "Possible causes:"
    echo "  1. Supabase project is paused - go to dashboard and resume"
    echo "  2. Wrong hostname in connection string"
    echo "  3. Network connectivity issues"
    echo ""
    echo "To fix:"
    echo "  1. Go to https://supabase.com/dashboard"
    echo "  2. Select your project"
    echo "  3. Go to Settings → Database"
    echo "  4. Copy the correct connection string"
    exit 1
fi

echo ""
echo "Testing database connection with SQLx..."
cargo run --quiet 2>&1 | head -20 &
SERVER_PID=$!
sleep 5
kill $SERVER_PID 2>/dev/null

echo ""
echo "========================================"
echo "Diagnostics complete"
echo "========================================"
