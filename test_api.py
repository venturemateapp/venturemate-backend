#!/usr/bin/env python3
"""
VentureMate API Testing Suite
Tests all endpoints systematically with dummy data
"""

import requests
import json
import uuid
import sys
from datetime import datetime
from typing import Dict, Any, Optional

# Configuration
BASE_URL = "http://127.0.0.1:8080/api/v1"
TEST_EMAIL = f"test_{uuid.uuid4().hex[:8]}@example.com"
TEST_PASSWORD = "TestPassword123!"
TEST_FIRST_NAME = "Test"
TEST_LAST_NAME = "User"

# Colors for terminal output
class Colors:
    GREEN = '\033[92m'
    RED = '\033[91m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    MAGENTA = '\033[95m'
    END = '\033[0m'
    BOLD = '\033[1m'

# Global state
access_token: Optional[str] = None
refresh_token: Optional[str] = None
user_id: Optional[str] = None
session_id: Optional[str] = None

# Test results
test_results: Dict[str, list] = {
    "passed": [],
    "failed": [],
    "skipped": []
}

def print_header(text: str):
    print(f"\n{Colors.BOLD}{Colors.CYAN}{'='*70}{Colors.END}")
    print(f"{Colors.BOLD}{Colors.CYAN} {text}{Colors.END}")
    print(f"{Colors.BOLD}{Colors.CYAN}{'='*70}{Colors.END}\n")

def print_subheader(text: str):
    print(f"\n{Colors.BOLD}{Colors.BLUE}{text}{Colors.END}")
    print(f"{Colors.BLUE}{'-'*50}{Colors.END}")

def log_request(method: str, endpoint: str, data: Any = None):
    print(f"\n{Colors.YELLOW}▶ {method} {endpoint}{Colors.END}")
    if data:
        print(f"{Colors.YELLOW}  Request Body:{Colors.END}")
        print(json.dumps(data, indent=2))

def log_response(response: requests.Response, expected_status: int = 200):
    status_color = Colors.GREEN if response.status_code == expected_status else Colors.RED
    print(f"{status_color}  Status: {response.status_code}{Colors.END}")
    try:
        body = response.json()
        print(f"{Colors.CYAN}  Response:{Colors.END}")
        print(json.dumps(body, indent=2)[:500] + "..." if len(json.dumps(body)) > 500 else json.dumps(body, indent=2))
        return body
    except:
        print(f"{Colors.CYAN}  Response: {response.text[:200]}{Colors.END}")
        return None

def test_passed(name: str):
    test_results["passed"].append(name)
    print(f"\n{Colors.GREEN}✓ PASSED: {name}{Colors.END}")

def test_failed(name: str, error: str):
    test_results["failed"].append((name, error))
    print(f"\n{Colors.RED}✗ FAILED: {name}{Colors.END}")
    print(f"{Colors.RED}  Error: {error}{Colors.END}")

def test_skipped(name: str, reason: str):
    test_results["skipped"].append((name, reason))
    print(f"\n{Colors.YELLOW}⊘ SKIPPED: {name} - {reason}{Colors.END}")

def get_headers(auth: bool = False) -> Dict[str, str]:
    headers = {"Content-Type": "application/json"}
    if auth and access_token:
        headers["Authorization"] = f"Bearer {access_token}"
    return headers

# =============================================================================
# HEALTH MODULE TESTS
# =============================================================================

def test_health():
    print_subheader("🔍 Testing Health Endpoint")
    try:
        log_request("GET", "/health")
        response = requests.get(f"{BASE_URL}/health", timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success"):
            test_passed("Health check endpoint")
        else:
            test_failed("Health check endpoint", f"Unexpected response: {body}")
    except Exception as e:
        test_failed("Health check endpoint", str(e))

# =============================================================================
# AUTH MODULE TESTS
# =============================================================================

def test_auth_register():
    """Test user registration"""
    print_subheader("🔐 Testing User Registration")
    global user_id
    
    try:
        data = {
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
            "first_name": TEST_FIRST_NAME,
            "last_name": TEST_LAST_NAME,
            "country_code": "US"
        }
        log_request("POST", "/auth/register", data)
        response = requests.post(f"{BASE_URL}/auth/register", json=data, timeout=10)
        body = log_response(response, 201)
        
        if response.status_code == 201 and body and body.get("success"):
            user_id = body["data"]["user"]["id"]
            test_passed("User registration")
        elif response.status_code == 409:
            test_skipped("User registration", "Email already exists (trying login instead)")
        else:
            test_failed("User registration", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("User registration", str(e))

def test_auth_login():
    """Test user login"""
    print_subheader("🔐 Testing User Login")
    global access_token, refresh_token, user_id
    
    try:
        data = {
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD,
            "remember_me": True
        }
        log_request("POST", "/auth/login", data)
        response = requests.post(f"{BASE_URL}/auth/login", json=data, timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success"):
            access_token = body["data"]["tokens"]["access_token"]
            refresh_token = body["data"]["tokens"]["refresh_token"]
            user_id = body["data"]["user"]["id"]
            test_passed("User login")
        else:
            test_failed("User login", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("User login", str(e))

def test_auth_status():
    """Test auth status check"""
    print_subheader("🔐 Testing Auth Status")
    
    try:
        log_request("GET", "/auth/status")
        response = requests.get(f"{BASE_URL}/auth/status", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Auth status check")
        else:
            test_failed("Auth status check", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Auth status check", str(e))

def test_auth_refresh_token():
    """Test token refresh"""
    print_subheader("🔐 Testing Token Refresh")
    global access_token
    
    if not refresh_token:
        test_skipped("Token refresh", "No refresh token available")
        return
    
    try:
        data = {"refresh_token": refresh_token}
        log_request("POST", "/auth/refresh", data)
        response = requests.post(f"{BASE_URL}/auth/refresh", json=data, timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success"):
            access_token = body["data"]["access_token"]
            test_passed("Token refresh")
        else:
            test_failed("Token refresh", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("Token refresh", str(e))

def test_auth_forgot_password():
    """Test forgot password"""
    print_subheader("🔐 Testing Forgot Password")
    
    try:
        data = {"email": TEST_EMAIL}
        log_request("POST", "/auth/forgot-password", data)
        response = requests.post(f"{BASE_URL}/auth/forgot-password", json=data, timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Forgot password request")
        else:
            test_failed("Forgot password request", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Forgot password request", str(e))

def test_auth_google_url():
    """Test Google OAuth URL generation"""
    print_subheader("🔐 Testing Google OAuth URL")
    
    try:
        log_request("GET", "/auth/google")
        response = requests.get(f"{BASE_URL}/auth/google", timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success") and "auth_url" in body.get("data", {}):
            test_passed("Google OAuth URL generation")
        else:
            test_failed("Google OAuth URL generation", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("Google OAuth URL generation", str(e))

def test_auth_resend_verification():
    """Test resend verification email"""
    print_subheader("🔐 Testing Resend Verification Email")
    
    try:
        data = {"email": TEST_EMAIL}
        log_request("POST", "/auth/resend-verification", data)
        response = requests.post(f"{BASE_URL}/auth/resend-verification", json=data, timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Resend verification email")
        else:
            test_failed("Resend verification email", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Resend verification email", str(e))

# =============================================================================
# USERS MODULE TESTS
# =============================================================================

def test_users_get_profile():
    """Test get user profile"""
    print_subheader("👤 Testing Get User Profile")
    
    if not access_token:
        test_skipped("Get user profile", "No access token available")
        return
    
    try:
        log_request("GET", "/users/me")
        response = requests.get(f"{BASE_URL}/users/me", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success"):
            test_passed("Get user profile")
        else:
            test_failed("Get user profile", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("Get user profile", str(e))

def test_users_update_profile():
    """Test update user profile"""
    print_subheader("👤 Testing Update User Profile")
    
    if not access_token:
        test_skipped("Update user profile", "No access token available")
        return
    
    try:
        data = {
            "first_name": "Updated",
            "last_name": "Name",
            "phone": "+1234567890",
            "job_title": "Founder",
            "company_name": "Test Startup"
        }
        log_request("PUT", "/users/me", data)
        response = requests.put(f"{BASE_URL}/users/me", json=data, headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200 and body and body.get("success"):
            test_passed("Update user profile")
        else:
            test_failed("Update user profile", f"Status: {response.status_code}, Body: {body}")
    except Exception as e:
        test_failed("Update user profile", str(e))

def test_users_list_sessions():
    """Test list user sessions"""
    print_subheader("👤 Testing List User Sessions")
    
    if not access_token:
        test_skipped("List user sessions", "No access token available")
        return
    
    try:
        log_request("GET", "/users/me/sessions")
        response = requests.get(f"{BASE_URL}/users/me/sessions", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("List user sessions")
        else:
            test_failed("List user sessions", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("List user sessions", str(e))

def test_users_list():
    """Test list users"""
    print_subheader("👤 Testing List Users")
    
    if not access_token:
        test_skipped("List users", "No access token available")
        return
    
    try:
        log_request("GET", "/users?page=1&per_page=10")
        response = requests.get(f"{BASE_URL}/users?page=1&per_page=10", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("List users")
        else:
            test_failed("List users", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("List users", str(e))

# =============================================================================
# DASHBOARD MODULE TESTS
# =============================================================================

def test_dashboard_get():
    """Test get dashboard"""
    print_subheader("📊 Testing Get Dashboard")
    
    if not access_token:
        test_skipped("Get dashboard", "No access token available")
        return
    
    try:
        log_request("GET", "/dashboard")
        response = requests.get(f"{BASE_URL}/dashboard", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Get dashboard")
        else:
            test_failed("Get dashboard", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get dashboard", str(e))

def test_dashboard_quick_actions():
    """Test get quick actions"""
    print_subheader("📊 Testing Get Quick Actions")
    
    if not access_token:
        test_skipped("Get quick actions", "No access token available")
        return
    
    try:
        log_request("GET", "/dashboard/quick-actions")
        response = requests.get(f"{BASE_URL}/dashboard/quick-actions", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Get quick actions")
        else:
            test_failed("Get quick actions", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get quick actions", str(e))

def test_dashboard_activity_feed():
    """Test get activity feed"""
    print_subheader("📊 Testing Get Activity Feed")
    
    if not access_token:
        test_skipped("Get activity feed", "No access token available")
        return
    
    try:
        log_request("GET", "/dashboard/activity-feed")
        response = requests.get(f"{BASE_URL}/dashboard/activity-feed", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Get activity feed")
        else:
            test_failed("Get activity feed", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get activity feed", str(e))

# =============================================================================
# ONBOARDING MODULE TESTS
# =============================================================================

def test_onboarding_get_status():
    """Test get onboarding status"""
    print_subheader("🏢 Testing Get Onboarding Status")
    
    if not access_token:
        test_skipped("Get onboarding status", "No access token available")
        return
    
    try:
        log_request("GET", "/onboarding/status")
        response = requests.get(f"{BASE_URL}/onboarding/status", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Get onboarding status")
        else:
            test_failed("Get onboarding status", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get onboarding status", str(e))

# =============================================================================
# BUSINESS MODULE TESTS
# =============================================================================

def test_businesses_list():
    """Test list businesses"""
    print_subheader("💼 Testing List Businesses")
    
    if not access_token:
        test_skipped("List businesses", "No access token available")
        return
    
    try:
        log_request("GET", "/businesses")
        response = requests.get(f"{BASE_URL}/businesses", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("List businesses")
        else:
            test_failed("List businesses", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("List businesses", str(e))

# =============================================================================
# SUBSCRIPTIONS MODULE TESTS
# =============================================================================

def test_subscriptions_get_current():
    """Test get current subscription"""
    print_subheader("💳 Testing Get Current Subscription")
    
    if not access_token:
        test_skipped("Get current subscription", "No access token available")
        return
    
    try:
        log_request("GET", "/subscriptions/current")
        response = requests.get(f"{BASE_URL}/subscriptions/current", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        # 404 is OK if no subscription exists
        if response.status_code in [200, 404]:
            test_passed("Get current subscription")
        else:
            test_failed("Get current subscription", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get current subscription", str(e))

def test_subscriptions_get_plans():
    """Test get subscription plans"""
    print_subheader("💳 Testing Get Subscription Plans")
    
    if not access_token:
        test_skipped("Get subscription plans", "No access token available")
        return
    
    try:
        log_request("GET", "/subscriptions/plans")
        response = requests.get(f"{BASE_URL}/subscriptions/plans", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("Get subscription plans")
        else:
            test_failed("Get subscription plans", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("Get subscription plans", str(e))

# =============================================================================
# DOCUMENTS MODULE TESTS
# =============================================================================

def test_documents_list():
    """Test list documents"""
    print_subheader("📄 Testing List Documents")
    
    if not access_token:
        test_skipped("List documents", "No access token available")
        return
    
    try:
        log_request("GET", "/documents")
        response = requests.get(f"{BASE_URL}/documents", headers=get_headers(auth=True), timeout=10)
        body = log_response(response, 200)
        
        if response.status_code == 200:
            test_passed("List documents")
        else:
            test_failed("List documents", f"Status: {response.status_code}")
    except Exception as e:
        test_failed("List documents", str(e))

# =============================================================================
# MAIN EXECUTION
# =============================================================================

def run_auth_tests():
    """Run all authentication tests"""
    print_header("🔐 AUTH MODULE TESTS")
    
    test_health()
    test_auth_register()
    test_auth_login()
    test_auth_status()
    test_auth_refresh_token()
    test_auth_forgot_password()
    test_auth_google_url()
    test_auth_resend_verification()

def run_user_tests():
    """Run all user tests"""
    print_header("👤 USERS MODULE TESTS")
    
    test_users_get_profile()
    test_users_update_profile()
    test_users_list_sessions()
    test_users_list()

def run_dashboard_tests():
    """Run all dashboard tests"""
    print_header("📊 DASHBOARD MODULE TESTS")
    
    test_dashboard_get()
    test_dashboard_quick_actions()
    test_dashboard_activity_feed()

def run_onboarding_tests():
    """Run all onboarding tests"""
    print_header("🏢 ONBOARDING MODULE TESTS")
    
    test_onboarding_get_status()

def run_business_tests():
    """Run all business tests"""
    print_header("💼 BUSINESSES MODULE TESTS")
    
    test_businesses_list()

def run_subscription_tests():
    """Run all subscription tests"""
    print_header("💳 SUBSCRIPTIONS MODULE TESTS")
    
    test_subscriptions_get_current()
    test_subscriptions_get_plans()

def run_document_tests():
    """Run all document tests"""
    print_header("📄 DOCUMENTS MODULE TESTS")
    
    test_documents_list()

def print_summary():
    """Print test summary"""
    print_header("📊 TEST SUMMARY")
    
    total = len(test_results["passed"]) + len(test_results["failed"]) + len(test_results["skipped"])
    
    print(f"\n{Colors.BOLD}Total Tests: {total}{Colors.END}")
    print(f"{Colors.GREEN}✓ Passed: {len(test_results['passed'])}{Colors.END}")
    print(f"{Colors.RED}✗ Failed: {len(test_results['failed'])}{Colors.END}")
    print(f"{Colors.YELLOW}⊘ Skipped: {len(test_results['skipped'])}{Colors.END}")
    
    if test_results["failed"]:
        print(f"\n{Colors.RED}{Colors.BOLD}Failed Tests:{Colors.END}")
        for name, error in test_results["failed"]:
            print(f"  {Colors.RED}• {name}: {error}{Colors.END}")
    
    if test_results["skipped"]:
        print(f"\n{Colors.YELLOW}{Colors.BOLD}Skipped Tests:{Colors.END}")
        for name, reason in test_results["skipped"]:
            print(f"  {Colors.YELLOW}• {name}: {reason}{Colors.END}")
    
    success_rate = (len(test_results["passed"]) / total * 100) if total > 0 else 0
    print(f"\n{Colors.BOLD}Success Rate: {success_rate:.1f}%{Colors.END}")
    
    return len(test_results["failed"]) == 0

def main():
    print(f"{Colors.BOLD}{Colors.MAGENTA}")
    print("╔══════════════════════════════════════════════════════════════╗")
    print("║           VentureMate API Testing Suite                      ║")
    print("╚══════════════════════════════════════════════════════════════╝")
    print(f"{Colors.END}")
    print(f"Test Email: {TEST_EMAIL}")
    print(f"Base URL: {BASE_URL}")
    
    # Run all tests
    run_auth_tests()
    run_user_tests()
    run_dashboard_tests()
    run_onboarding_tests()
    run_business_tests()
    run_subscription_tests()
    run_document_tests()
    
    # Print summary
    success = print_summary()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
