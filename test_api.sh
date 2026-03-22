#!/bin/bash

# VentureMate API Testing Suite
# Tests all endpoints systematically with dummy data

set -e

BASE_URL="http://127.0.0.1:8080/api/v1"
TEST_EMAIL="test_$(date +%s)@example.com"
TEST_PASSWORD="TestPassword123!"
TEST_FIRST_NAME="Test"
TEST_LAST_NAME="User"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Global state
ACCESS_TOKEN=""
REFRESH_TOKEN=""
USER_ID=""

# Helper functions
print_header() {
    echo -e "\n${BOLD}${CYAN}============================================================${NC}"
    echo -e "${BOLD}${CYAN} $1${NC}"
    echo -e "${BOLD}${CYAN}============================================================${NC}\n"
}

print_subheader() {
    echo -e "\n${BOLD}${BLUE}$1${NC}"
    echo -e "${BLUE}------------------------------------------------------------${NC}"
}

log_request() {
    local method=$1
    local endpoint=$2
    echo -e "\n${YELLOW}▶ $method $endpoint${NC}"
}

test_passed() {
    ((PASSED++))
    echo -e "${GREEN}✓ PASSED: $1${NC}"
}

test_failed() {
    ((FAILED++))
    echo -e "${RED}✗ FAILED: $1${NC}"
    echo -e "${RED}  Error: $2${NC}"
}

test_skipped() {
    ((SKIPPED++))
    echo -e "${YELLOW}⊘ SKIPPED: $1 - $2${NC}"
}

# =============================================================================
# HEALTH TESTS
# =============================================================================

test_health() {
    print_subheader "🔍 Testing Health Endpoint"
    
    log_request "GET" "/health"
    RESPONSE=$(curl -s -w "\n%{http_code}" "${BASE_URL}/health" 2>/dev/null || echo "error")
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Health check endpoint"
        echo "  Response: $BODY"
    else
        test_failed "Health check endpoint" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# AUTH TESTS
# =============================================================================

test_auth_register() {
    print_subheader "🔐 Testing User Registration"
    
    JSON_DATA=$(cat <<EOF
{
    "email": "$TEST_EMAIL",
    "password": "$TEST_PASSWORD",
    "first_name": "$TEST_FIRST_NAME",
    "last_name": "$TEST_LAST_NAME",
    "country_code": "US"
}
EOF
)
    
    log_request "POST" "/auth/register"
    echo "  Request Body: $JSON_DATA"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$JSON_DATA" \
        "${BASE_URL}/auth/register" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "201" ]; then
        test_passed "User registration"
        USER_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo "  Created User ID: $USER_ID"
    elif [ "$HTTP_CODE" = "409" ]; then
        test_skipped "User registration" "Email already exists"
    else
        test_failed "User registration" "HTTP $HTTP_CODE: $BODY"
    fi
}

test_auth_login() {
    print_subheader "🔐 Testing User Login"
    
    JSON_DATA=$(cat <<EOF
{
    "email": "$TEST_EMAIL",
    "password": "$TEST_PASSWORD",
    "remember_me": true
}
EOF
)
    
    log_request "POST" "/auth/login"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$JSON_DATA" \
        "${BASE_URL}/auth/login" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "User login"
        ACCESS_TOKEN=$(echo "$BODY" | grep -o '"access_token":"[^"]*"' | head -1 | cut -d'"' -f4)
        REFRESH_TOKEN=$(echo "$BODY" | grep -o '"refresh_token":"[^"]*"' | head -1 | cut -d'"' -f4)
        USER_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
        echo "  Access Token: ${ACCESS_TOKEN:0:50}..."
    else
        test_failed "User login" "HTTP $HTTP_CODE: $BODY"
    fi
}

test_auth_status() {
    print_subheader "🔐 Testing Auth Status"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Auth status check" "No access token"
        return
    fi
    
    log_request "GET" "/auth/status"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/auth/status" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Auth status check"
    else
        test_failed "Auth status check" "HTTP $HTTP_CODE"
    fi
}

test_auth_refresh_token() {
    print_subheader "🔐 Testing Token Refresh"
    
    if [ -z "$REFRESH_TOKEN" ]; then
        test_skipped "Token refresh" "No refresh token"
        return
    fi
    
    JSON_DATA="{\"refresh_token\":\"$REFRESH_TOKEN\"}"
    
    log_request "POST" "/auth/refresh"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$JSON_DATA" \
        "${BASE_URL}/auth/refresh" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Token refresh"
        NEW_TOKEN=$(echo "$BODY" | grep -o '"access_token":"[^"]*"' | head -1 | cut -d'"' -f4)
        if [ -n "$NEW_TOKEN" ]; then
            ACCESS_TOKEN="$NEW_TOKEN"
            echo "  New access token received"
        fi
    else
        test_failed "Token refresh" "HTTP $HTTP_CODE: $BODY"
    fi
}

test_auth_forgot_password() {
    print_subheader "🔐 Testing Forgot Password"
    
    JSON_DATA="{\"email\":\"$TEST_EMAIL\"}"
    
    log_request "POST" "/auth/forgot-password"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$JSON_DATA" \
        "${BASE_URL}/auth/forgot-password" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Forgot password request"
    else
        test_failed "Forgot password request" "HTTP $HTTP_CODE"
    fi
}

test_auth_google_url() {
    print_subheader "🔐 Testing Google OAuth URL"
    
    log_request "GET" "/auth/google"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        "${BASE_URL}/auth/google" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Google OAuth URL generation"
        echo "  Response preview: ${BODY:0:100}..."
    else
        test_failed "Google OAuth URL generation" "HTTP $HTTP_CODE"
    fi
}

test_auth_resend_verification() {
    print_subheader "🔐 Testing Resend Verification Email"
    
    JSON_DATA="{\"email\":\"$TEST_EMAIL\"}"
    
    log_request "POST" "/auth/resend-verification"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$JSON_DATA" \
        "${BASE_URL}/auth/resend-verification" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Resend verification email"
    else
        test_failed "Resend verification email" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# USERS TESTS
# =============================================================================

test_users_get_profile() {
    print_subheader "👤 Testing Get User Profile"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get user profile" "No access token"
        return
    fi
    
    log_request "GET" "/users/me"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/users/me" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d'
    )
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get user profile"
    else
        test_failed "Get user profile" "HTTP $HTTP_CODE: $BODY"
    fi
}

test_users_update_profile() {
    print_subheader "👤 Testing Update User Profile"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Update user profile" "No access token"
        return
    fi
    
    JSON_DATA=$(cat <<EOF
{
    "first_name": "Updated",
    "last_name": "Name",
    "phone": "+1234567890",
    "job_title": "Founder",
    "company_name": "Test Startup"
}
EOF
)
    
    log_request "PUT" "/users/me"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X PUT \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        -d "$JSON_DATA" \
        "${BASE_URL}/users/me" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Update user profile"
    else
        test_failed "Update user profile" "HTTP $HTTP_CODE: $BODY"
    fi
}

test_users_list_sessions() {
    print_subheader "👤 Testing List User Sessions"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "List user sessions" "No access token"
        return
    fi
    
    log_request "GET" "/users/me/sessions"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/users/me/sessions" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "List user sessions"
    else
        test_failed "List user sessions" "HTTP $HTTP_CODE"
    fi
}

test_users_list() {
    print_subheader "👤 Testing List Users"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "List users" "No access token"
        return
    fi
    
    log_request "GET" "/users?page=1&per_page=10"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/users?page=1&per_page=10" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "List users"
    else
        test_failed "List users" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# DASHBOARD TESTS
# =============================================================================

test_dashboard_get() {
    print_subheader "📊 Testing Get Dashboard"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get dashboard" "No access token"
        return
    fi
    
    log_request "GET" "/dashboard"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/dashboard" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get dashboard"
    else
        test_failed "Get dashboard" "HTTP $HTTP_CODE"
    fi
}

test_dashboard_quick_actions() {
    print_subheader "📊 Testing Get Quick Actions"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get quick actions" "No access token"
        return
    fi
    
    log_request "GET" "/dashboard/quick-actions"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/dashboard/quick-actions" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get quick actions"
    else
        test_failed "Get quick actions" "HTTP $HTTP_CODE"
    fi
}

test_dashboard_activity_feed() {
    print_subheader "📊 Testing Get Activity Feed"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get activity feed" "No access token"
        return
    fi
    
    log_request "GET" "/dashboard/activity-feed"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/dashboard/activity-feed" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get activity feed"
    else
        test_failed "Get activity feed" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# ONBOARDING TESTS
# =============================================================================

test_onboarding_get_status() {
    print_subheader "🏢 Testing Get Onboarding Status"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get onboarding status" "No access token"
        return
    fi
    
    log_request "GET" "/onboarding/status"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/onboarding/status" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get onboarding status"
    else
        test_failed "Get onboarding status" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# BUSINESSES TESTS
# =============================================================================

test_businesses_list() {
    print_subheader "💼 Testing List Businesses"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "List businesses" "No access token"
        return
    fi
    
    log_request "GET" "/businesses"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/businesses" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "List businesses"
    else
        test_failed "List businesses" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# SUBSCRIPTIONS TESTS
# =============================================================================

test_subscriptions_get_current() {
    print_subheader "💳 Testing Get Current Subscription"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get current subscription" "No access token"
        return
    fi
    
    log_request "GET" "/subscriptions/current"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/subscriptions/current" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    # 404 is OK if no subscription exists
    if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "404" ]; then
        test_passed "Get current subscription"
    else
        test_failed "Get current subscription" "HTTP $HTTP_CODE"
    fi
}

test_subscriptions_get_plans() {
    print_subheader "💳 Testing Get Subscription Plans"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "Get subscription plans" "No access token"
        return
    fi
    
    log_request "GET" "/subscriptions/plans"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/subscriptions/plans" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "Get subscription plans"
    else
        test_failed "Get subscription plans" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# DOCUMENTS TESTS
# =============================================================================

test_documents_list() {
    print_subheader "📄 Testing List Documents"
    
    if [ -z "$ACCESS_TOKEN" ]; then
        test_skipped "List documents" "No access token"
        return
    fi
    
    log_request "GET" "/documents"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer $ACCESS_TOKEN" \
        "${BASE_URL}/documents" 2>/dev/null || echo "error")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    
    if [ "$HTTP_CODE" = "200" ]; then
        test_passed "List documents"
    else
        test_failed "List documents" "HTTP $HTTP_CODE"
    fi
}

# =============================================================================
# PRINT SUMMARY
# =============================================================================

print_summary() {
    print_header "📊 TEST SUMMARY"
    
    TOTAL=$((PASSED + FAILED + SKIPPED))
    
    echo -e "\n${BOLD}Total Tests: $TOTAL${NC}"
    echo -e "${GREEN}✓ Passed: $PASSED${NC}"
    echo -e "${RED}✗ Failed: $FAILED${NC}"
    echo -e "${YELLOW}⊘ Skipped: $SKIPPED${NC}"
    
    if [ $TOTAL -gt 0 ]; then
        SUCCESS_RATE=$((PASSED * 100 / TOTAL))
        echo -e "\n${BOLD}Success Rate: ${SUCCESS_RATE}%${NC}"
    fi
    
    if [ $FAILED -eq 0 ]; then
        echo -e "\n${GREEN}${BOLD}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}${BOLD}⚠️  Some tests failed!${NC}"
        return 1
    fi
}

# =============================================================================
# MAIN
# =============================================================================

echo -e "${BOLD}${MAGENTA}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           VentureMate API Testing Suite                      ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo "Test Email: $TEST_EMAIL"
echo "Base URL: $BASE_URL"

# Run tests
print_header "🔐 AUTH MODULE TESTS"
test_health
test_auth_register
test_auth_login
test_auth_status
test_auth_refresh_token
test_auth_forgot_password
test_auth_google_url
test_auth_resend_verification

print_header "👤 USERS MODULE TESTS"
test_users_get_profile
test_users_update_profile
test_users_list_sessions
test_users_list

print_header "📊 DASHBOARD MODULE TESTS"
test_dashboard_get
test_dashboard_quick_actions
test_dashboard_activity_feed

print_header "🏢 ONBOARDING MODULE TESTS"
test_onboarding_get_status

print_header "💼 BUSINESSES MODULE TESTS"
test_businesses_list

print_header "💳 SUBSCRIPTIONS MODULE TESTS"
test_subscriptions_get_current
test_subscriptions_get_plans

print_header "📄 DOCUMENTS MODULE TESTS"
test_documents_list

# Print summary
print_summary
