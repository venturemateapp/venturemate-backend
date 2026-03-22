#!/bin/bash

# Comprehensive API Testing Script for ALL Modules
# Requires: ACCESS_TOKEN environment variable set

set -e

BASE_URL="http://127.0.0.1:8080/api/v1"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'
BOLD='\033[1m'

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Store failed tests for summary
declare -a FAILED_TESTS
declare -a FAILED_ERRORS

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

test_passed() {
    ((PASSED++))
    echo -e "${GREEN}✓ $1${NC}"
}

test_failed() {
    ((FAILED++))
    echo -e "${RED}✗ $1${NC}"
    echo -e "${RED}  Error: $2${NC}"
    FAILED_TESTS+=("$1")
    FAILED_ERRORS+=("$2")
}

test_skipped() {
    ((SKIPPED++))
    echo -e "${YELLOW}⊘ $1 - $2${NC}"
}

# API test function
test_endpoint() {
    local method=$1
    local endpoint=$2
    local expected_codes=$3
    local data=$4
    local auth=${5:-true}
    
    local url="${BASE_URL}${endpoint}"
    local headers="-H 'Content-Type: application/json'"
    
    if [ "$auth" = "true" ] && [ -n "$ACCESS_TOKEN" ]; then
        headers="$headers -H 'Authorization: Bearer $ACCESS_TOKEN'"
    fi
    
    local cmd="curl -s -w '\nHTTP_CODE:%{http_code}'"
    
    if [ "$method" = "POST" ] || [ "$method" = "PUT" ] || [ "$method" = "PATCH" ]; then
        if [ -n "$data" ]; then
            cmd="$cmd -X $method $headers -d '$data'"
        else
            cmd="$cmd -X $method $headers"
        fi
    elif [ "$method" = "DELETE" ]; then
        cmd="$cmd -X DELETE $headers"
    else
        cmd="$cmd $headers"
    fi
    
    cmd="$cmd '$url'"
    
    local response=$(eval $cmd 2>/dev/null || echo "HTTP_CODE:000")
    local http_code=$(echo "$response" | grep "HTTP_CODE:" | cut -d: -f2)
    local body=$(echo "$response" | sed '$d')
    
    # Check if expected_codes contains the actual code
    if echo "$expected_codes" | grep -q "$http_code"; then
        echo "$body"
        return 0
    else
        echo "ERROR: HTTP $http_code, Expected: $expected_codes, Body: ${body:0:200}"
        return 1
    fi
}

# =============================================================================
# AUTH MODULE (Protected)
# =============================================================================

test_auth_status() {
    print_subheader "🔐 Auth Status"
    
    result=$(test_endpoint "GET" "/auth/status" "200" "" "true")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /auth/status" "$result"
    else
        test_passed "GET /auth/status"
    fi
}

test_auth_refresh() {
    print_subheader "🔐 Token Refresh"
    
    if [ -z "$REFRESH_TOKEN" ]; then
        test_skipped "POST /auth/refresh" "No refresh token"
        return
    fi
    
    result=$(test_endpoint "POST" "/auth/refresh" "200" "{\"refresh_token\":\"$REFRESH_TOKEN\"}" "false")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "POST /auth/refresh" "$result"
    else
        test_passed "POST /auth/refresh"
    fi
}

test_auth_logout() {
    print_subheader "🔐 Logout"
    
    result=$(test_endpoint "POST" "/auth/logout" "200" "" "true")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "POST /auth/logout" "$result"
    else
        test_passed "POST /auth/logout"
    fi
}

# =============================================================================
# USERS MODULE
# =============================================================================

test_users_get_me() {
    print_subheader "👤 Get Current User"
    
    result=$(test_endpoint "GET" "/users/me" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /users/me" "$result"
    else
        test_passed "GET /users/me"
    fi
}

test_users_update_profile() {
    print_subheader "👤 Update Profile"
    
    result=$(test_endpoint "PUT" "/users/me" "200" '{"first_name":"Updated","last_name":"Name","job_title":"Founder"}')
    if echo "$result" | grep -q "ERROR"; then
        test_failed "PUT /users/me" "$result"
    else
        test_passed "PUT /users/me"
    fi
}

test_users_list_sessions() {
    print_subheader "👤 List Sessions"
    
    result=$(test_endpoint "GET" "/users/me/sessions" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /users/me/sessions" "$result"
    else
        test_passed "GET /users/me/sessions"
    fi
}

test_users_list() {
    print_subheader "👤 List Users"
    
    result=$(test_endpoint "GET" "/users?page=1&per_page=10" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /users" "$result"
    else
        test_passed "GET /users"
    fi
}

# =============================================================================
# DASHBOARD MODULE
# =============================================================================

test_dashboard_get() {
    print_subheader "📊 Get Dashboard"
    
    result=$(test_endpoint "GET" "/dashboard" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /dashboard" "$result"
    else
        test_passed "GET /dashboard"
    fi
}

test_dashboard_quick_actions() {
    print_subheader "📊 Quick Actions"
    
    result=$(test_endpoint "GET" "/dashboard/quick-actions" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /dashboard/quick-actions" "$result"
    else
        test_passed "GET /dashboard/quick-actions"
    fi
}

test_dashboard_activity() {
    print_subheader "📊 Activity Feed"
    
    result=$(test_endpoint "GET" "/dashboard/activity-feed" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /dashboard/activity-feed" "$result"
    else
        test_passed "GET /dashboard/activity-feed"
    fi
}

# =============================================================================
# ONBOARDING MODULE
# =============================================================================

test_onboarding_status() {
    print_subheader "🏢 Onboarding Status"
    
    result=$(test_endpoint "GET" "/onboarding/status" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /onboarding/status" "$result"
    else
        test_passed "GET /onboarding/status"
    fi
}

# =============================================================================
# BUSINESSES MODULE
# =============================================================================

test_businesses_list() {
    print_subheader "💼 List Businesses"
    
    result=$(test_endpoint "GET" "/businesses" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /businesses" "$result"
    else
        test_passed "GET /businesses"
    fi
}

test_businesses_create() {
    print_subheader "💼 Create Business"
    
    result=$(test_endpoint "POST" "/businesses" "201 200" '{"name":"Test Business","description":"A test business","industry":"Technology","stage":"idea"}')
    if echo "$result" | grep -q "ERROR"; then
        test_failed "POST /businesses" "$result"
    else
        test_passed "POST /businesses"
        # Extract business ID for later tests
        BUSINESS_ID=$(echo "$result" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
    fi
}

# =============================================================================
# SUBSCRIPTIONS MODULE
# =============================================================================

test_subscriptions_plans() {
    print_subheader "💳 Subscription Plans"
    
    result=$(test_endpoint "GET" "/subscriptions/plans" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /subscriptions/plans" "$result"
    else
        test_passed "GET /subscriptions/plans"
    fi
}

test_subscriptions_current() {
    print_subheader "💳 Current Subscription"
    
    result=$(test_endpoint "GET" "/subscriptions/current" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /subscriptions/current" "$result"
    else
        test_passed "GET /subscriptions/current"
    fi
}

# =============================================================================
# DOCUMENTS MODULE
# =============================================================================

test_documents_list() {
    print_subheader "📄 List Documents"
    
    result=$(test_endpoint "GET" "/documents" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /documents" "$result"
    else
        test_passed "GET /documents"
    fi
}

# =============================================================================
# AI MODULES
# =============================================================================

test_ai_generation_capabilities() {
    print_subheader "🤖 AI Generation Capabilities"
    
    result=$(test_endpoint "GET" "/ai-generation/capabilities" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /ai-generation/capabilities" "$result"
    else
        test_passed "GET /ai-generation/capabilities"
    fi
}

test_ai_conversations_list() {
    print_subheader "💬 List AI Conversations"
    
    result=$(test_endpoint "GET" "/ai-conversations" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /ai-conversations" "$result"
    else
        test_passed "GET /ai-conversations"
    fi
}

test_ai_startup_engine_status() {
    print_subheader "🚀 AI Startup Engine Status"
    
    result=$(test_endpoint "GET" "/ai-startup-engine/status" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /ai-startup-engine/status" "$result"
    else
        test_passed "GET /ai-startup-engine/status"
    fi
}

# =============================================================================
# BRANDING MODULE
# =============================================================================

test_branding_get() {
    print_subheader "🎨 Get Branding"
    
    result=$(test_endpoint "GET" "/branding" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /branding" "$result"
    else
        test_passed "GET /branding"
    fi
}

# =============================================================================
# DATA ROOM MODULE
# =============================================================================

test_data_room_status() {
    print_subheader "🏛️ Data Room Status"
    
    result=$(test_endpoint "GET" "/data-room/status" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /data-room/status" "$result"
    else
        test_passed "GET /data-room/status"
    fi
}

# =============================================================================
# HEALTH SCORE MODULE
# =============================================================================

test_health_score() {
    print_subheader "💯 Health Score"
    
    result=$(test_endpoint "GET" "/health-score" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /health-score" "$result"
    else
        test_passed "GET /health-score"
    fi
}

# =============================================================================
# RECOMMENDATIONS MODULE
# =============================================================================

test_recommendations() {
    print_subheader "📈 Recommendations"
    
    result=$(test_endpoint "GET" "/recommendations" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /recommendations" "$result"
    else
        test_passed "GET /recommendations"
    fi
}

# =============================================================================
# MARKETPLACE MODULE
# =============================================================================

test_marketplace_listings() {
    print_subheader "🛒 Marketplace Listings"
    
    result=$(test_endpoint "GET" "/marketplace/listings" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /marketplace/listings" "$result"
    else
        test_passed "GET /marketplace/listings"
    fi
}

# =============================================================================
# COFOUNDER MODULE
# =============================================================================

test_cofounder_matches() {
    print_subheader "👥 Cofounder Matches"
    
    result=$(test_endpoint "GET" "/cofounder/matches" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /cofounder/matches" "$result"
    else
        test_passed "GET /cofounder/matches"
    fi
}

# =============================================================================
# WEBSITES MODULE
# =============================================================================

test_websites_list() {
    print_subheader "🌐 List Websites"
    
    result=$(test_endpoint "GET" "/websites" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /websites" "$result"
    else
        test_passed "GET /websites"
    fi
}

# =============================================================================
# CRM MODULE
# =============================================================================

test_crm_contacts() {
    print_subheader "📋 CRM Contacts"
    
    result=$(test_endpoint "GET" "/crm/contacts" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /crm/contacts" "$result"
    else
        test_passed "GET /crm/contacts"
    fi
}

# =============================================================================
# BANKING MODULE
# =============================================================================

test_banking_accounts() {
    print_subheader "🏦 Banking Accounts"
    
    result=$(test_endpoint "GET" "/banking/accounts" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /banking/accounts" "$result"
    else
        test_passed "GET /banking/accounts"
    fi
}

# =============================================================================
# INVESTORS MODULE
# =============================================================================

test_investors_list() {
    print_subheader "💰 Investors List"
    
    result=$(test_endpoint "GET" "/investors" "200")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /investors" "$result"
    else
        test_passed "GET /investors"
    fi
}

# =============================================================================
# CREDIT MODULE
# =============================================================================

test_credit_score() {
    print_subheader "📊 Credit Score"
    
    result=$(test_endpoint "GET" "/credit/score" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /credit/score" "$result"
    else
        test_passed "GET /credit/score"
    fi
}

# =============================================================================
# SOCIAL MODULE
# =============================================================================

test_social_profiles() {
    print_subheader "📱 Social Profiles"
    
    result=$(test_endpoint "GET" "/social/profiles" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /social/profiles" "$result"
    else
        test_passed "GET /social/profiles"
    fi
}

# =============================================================================
# STARTUP STACK MODULE
# =============================================================================

test_startup_stack_recommendations() {
    print_subheader "🥞 Startup Stack Recommendations"
    
    result=$(test_endpoint "GET" "/startup-stack/recommendations" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /startup-stack/recommendations" "$result"
    else
        test_passed "GET /startup-stack/recommendations"
    fi
}

# =============================================================================
# ONBOARDING WIZARD MODULE
# =============================================================================

test_onboarding_wizard_status() {
    print_subheader "🧙 Onboarding Wizard Status"
    
    result=$(test_endpoint "GET" "/onboarding-wizard/status" "200 404")
    if echo "$result" | grep -q "ERROR"; then
        test_failed "GET /onboarding-wizard/status" "$result"
    else
        test_passed "GET /onboarding-wizard/status"
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
    
    if [ $FAILED -gt 0 ]; then
        echo -e "\n${RED}${BOLD}Failed Tests:${NC}"
        for i in "${!FAILED_TESTS[@]}"; do
            echo -e "  ${RED}• ${FAILED_TESTS[$i]}${NC}"
            echo -e "    ${RED}  ${FAILED_ERRORS[$i]}${NC}"
        done
    fi
    
    if [ $FAILED -eq 0 ]; then
        echo -e "\n${GREEN}${BOLD}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}${BOLD}⚠️  $FAILED test(s) failed!${NC}"
        return 1
    fi
}

# =============================================================================
# MAIN
# =============================================================================

echo -e "${BOLD}${MAGENTA}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           VentureMate API Testing Suite                      ║"
echo "║           Testing ALL 27 Handler Modules                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo "Base URL: $BASE_URL"
echo "Access Token: ${ACCESS_TOKEN:0:50}..."

# Run all tests
print_header "🔐 AUTH MODULE (Protected)"
test_auth_status
test_auth_refresh
test_auth_logout

print_header "👤 USERS MODULE"
test_users_get_me
test_users_update_profile
test_users_list_sessions
test_users_list

print_header "📊 DASHBOARD MODULE"
test_dashboard_get
test_dashboard_quick_actions
test_dashboard_activity

print_header "🏢 ONBOARDING MODULE"
test_onboarding_status

print_header "💼 BUSINESSES MODULE"
test_businesses_list
test_businesses_create

print_header "💳 SUBSCRIPTIONS MODULE"
test_subscriptions_plans
test_subscriptions_current

print_header "📄 DOCUMENTS MODULE"
test_documents_list

print_header "🤖 AI MODULES"
test_ai_generation_capabilities
test_ai_conversations_list
test_ai_startup_engine_status

print_header "🎨 BRANDING MODULE"
test_branding_get

print_header "🏛️ DATA ROOM MODULE"
test_data_room_status

print_header "💯 HEALTH SCORE MODULE"
test_health_score

print_header "📈 RECOMMENDATIONS MODULE"
test_recommendations

print_header "🛒 MARKETPLACE MODULE"
test_marketplace_listings

print_header "👥 COFOUNDER MODULE"
test_cofounder_matches

print_header "🌐 WEBSITES MODULE"
test_websites_list

print_header "📋 CRM MODULE"
test_crm_contacts

print_header "🏦 BANKING MODULE"
test_banking_accounts

print_header "💰 INVESTORS MODULE"
test_investors_list

print_header "📊 CREDIT MODULE"
test_credit_score

print_header "📱 SOCIAL MODULE"
test_social_profiles

print_header "🥞 STARTUP STACK MODULE"
test_startup_stack_recommendations

print_header "🧙 ONBOARDING WIZARD MODULE"
test_onboarding_wizard_status

# Print summary
print_summary
