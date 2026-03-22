# VentureMate - Complete API Specification

> RESTful API Documentation for VentureMate Platform

## 📋 Table of Contents

1. [Overview](#1-overview)
2. [Authentication](#2-authentication)
3. [User Management](#3-user-management)
4. [Onboarding](#4-onboarding)
5. [Business Management](#5-business-management)
6. [AI Generation Services](#6-ai-generation-services)
7. [Branding & Media](#7-branding--media)
8. [Website Builder](#8-website-builder)
9. [Document Vault](#9-document-vault)
10. [Marketplace](#10-marketplace)
11. [Subscriptions & Billing](#11-subscriptions--billing)
12. [Analytics & Health Score](#12-analytics--health-score)
13. [Webhooks](#13-webhooks)
14. [Error Handling](#14-error-handling)

---

## 1. Overview

### Base URL

```
Production:  https://api.venturemate.co/v1
Staging:     https://api-staging.venturemate.co/v1
Local:       http://localhost:8080/api/v1
```

### Content-Type

All requests should include:
```
Content-Type: application/json
Authorization: Bearer <jwt_token>
```

### Response Format

```json
{
  "success": true,
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 100
  },
  "error": null
}
```

### Error Format

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": [
      { "field": "email", "message": "Invalid email format" }
    ]
  }
}
```

---

## 2. Authentication

### 2.1 Register with Email

```http
POST /auth/register
```

**Request Body:**
```json
{
  "email": "founder@example.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe",
  "country_code": "ZA"
}
```

**Response (201 Created):**
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "founder@example.com",
      "first_name": "John",
      "last_name": "Doe",
      "email_verified": false,
      "created_at": "2025-03-20T10:00:00Z"
    },
    "tokens": {
      "access_token": "eyJhbGciOiJIUzI1NiIs...",
      "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
      "expires_in": 900
    }
  }
}
```

### 2.2 Login with Email

```http
POST /auth/login
```

**Request Body:**
```json
{
  "email": "founder@example.com",
  "password": "SecurePass123!"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "user": { ... },
    "tokens": {
      "access_token": "eyJhbGciOiJIUzI1NiIs...",
      "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
      "expires_in": 900
    }
  }
}
```

### 2.3 Google OAuth Login

```http
POST /auth/oauth/google
```

**Request Body:**
```json
{
  "id_token": "google_id_token_here",
  "access_token": "google_access_token_here"
}
```

### 2.4 Refresh Token

```http
POST /auth/refresh
```

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### 2.5 Logout

```http
POST /auth/logout
Authorization: Bearer <token>
```

### 2.6 Request Password Reset

```http
POST /auth/password-reset-request
```

**Request Body:**
```json
{
  "email": "founder@example.com"
}
```

### 2.7 Confirm Password Reset

```http
POST /auth/password-reset
```

**Request Body:**
```json
{
  "token": "reset_token_from_email",
  "new_password": "NewSecurePass123!"
}
```

### 2.8 Verify Email

```http
POST /auth/verify-email
```

**Request Body:**
```json
{
  "token": "verification_token_from_email"
}
```

---

## 3. User Management

### 3.1 Get Current User

```http
GET /users/me
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "founder@example.com",
    "first_name": "John",
    "last_name": "Doe",
    "avatar_url": "https://...",
    "email_verified": true,
    "phone": "+27xxxxxxxxx",
    "country_code": "ZA",
    "timezone": "Africa/Johannesburg",
    "subscription_tier": "starter",
    "created_at": "2025-03-20T10:00:00Z",
    "updated_at": "2025-03-20T10:00:00Z",
    "onboarding_completed": true,
    "businesses_count": 2
  }
}
```

### 3.2 Update Profile

```http
PATCH /users/me
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "first_name": "John",
  "last_name": "Smith",
  "phone": "+27123456789",
  "timezone": "Africa/Lagos"
}
```

### 3.3 Upload Avatar

```http
POST /users/me/avatar
Authorization: Bearer <token>
Content-Type: multipart/form-data
```

**Request Body:**
```
file: <binary_image_data>
```

### 3.4 Change Password

```http
POST /users/me/change-password
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "current_password": "OldPass123!",
  "new_password": "NewPass123!"
}
```

### 3.5 Delete Account

```http
DELETE /users/me
Authorization: Bearer <token>
```

---

## 4. Onboarding

### 4.1 Start Onboarding Session

```http
POST /onboarding/start
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session_id": "uuid",
    "current_step": "idea_intake",
    "progress_percentage": 0,
    "steps": [
      { "id": "idea_intake", "name": "Business Idea", "status": "in_progress" },
      { "id": "founder_profile", "name": "Founder Profile", "status": "pending" },
      { "id": "business_details", "name": "Business Details", "status": "pending" },
      { "id": "review", "name": "Review & Generate", "status": "pending" }
    ]
  }
}
```

### 4.2 Submit Idea Intake

```http
POST /onboarding/idea-intake
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "session_id": "uuid",
  "business_idea": "A mobile app that connects farmers directly with urban consumers, eliminating middlemen and ensuring fair prices for fresh produce",
  "problem_statement": "Farmers lose 40% of income to middlemen, consumers pay high prices",
  "target_customers": "Urban families, restaurants, hotels",
  "country_code": "NG",
  "city": "Lagos",
  "founder_type": "solo",
  "team_size": 1,
  "has_cofounder": false
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session_id": "uuid",
    "ai_analysis": {
      "industry": "AgriTech / Food Delivery",
      "sub_industry": "Farm-to-Table Marketplace",
      "market_size": "Large - $12B African market",
      "complexity": "medium",
      "estimated_launch_time": "6-8 weeks",
      "suggested_business_models": [
        "Commission on sales",
        "Subscription for farmers",
        "Delivery fees"
      ]
    },
    "next_step": "founder_profile",
    "progress_percentage": 25
  }
}
```

### 4.3 Submit Founder Profile

```http
POST /onboarding/founder-profile
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "session_id": "uuid",
  "experience_level": "first_time",
  "background": "software_engineer",
  "skills": ["programming", "product_management"],
  "availability": "full_time",
  "funding_preference": "bootstrap",
  "motivation": "solve_real_problem",
  "challenges": ["marketing", "fundraising"]
}
```

### 4.4 Submit Business Details

```http
POST /onboarding/business-details
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "session_id": "uuid",
  "preferred_business_name": "FarmConnect Nigeria",
  "alternative_names": ["AgriDirect", "FreshLink NG"],
  "business_model": "commission",
  "revenue_streams": ["commission", "delivery_fees", "subscriptions"],
  "initial_funding": 500000,
  "currency": "NGN",
  "timeline": "1_month",
  "legal_structure_preference": "ltd"
}
```

### 4.5 Review & Generate

```http
POST /onboarding/review
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "session_id": "uuid",
  "confirmed": true,
  "generate_options": {
    "business_plan": true,
    "branding_kit": true,
    "website": true,
    "pitch_deck": false
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "business_id": "uuid",
    "generation_jobs": [
      { "job_id": "uuid", "type": "business_plan", "status": "queued" },
      { "job_id": "uuid", "type": "branding_kit", "status": "queued" },
      { "job_id": "uuid", "type": "website", "status": "queued" }
    ],
    "estimated_completion": "2025-03-20T10:05:00Z",
    "dashboard_url": "/dashboard/business/uuid"
  }
}
```

### 4.6 Get Onboarding Status

```http
GET /onboarding/status/:session_id
Authorization: Bearer <token>
```

---

## 5. Business Management

### 5.1 List My Businesses

```http
GET /businesses
Authorization: Bearer <token>
```

**Query Parameters:**
- `page` (int, default: 1)
- `per_page` (int, default: 20, max: 100)
- `status` (string, optional): active, draft, archived

**Response:**
```json
{
  "success": true,
  "data": {
    "businesses": [
      {
        "id": "uuid",
        "name": "FarmConnect Nigeria",
        "slug": "farmconnect-nigeria",
        "industry": "AgriTech",
        "status": "active",
        "stage": "idea",
        "country": "NG",
        "health_score": 72,
        "logo_url": "https://...",
        "website_url": "https://farmconnect-ng.venture.site",
        "created_at": "2025-03-20T10:00:00Z",
        "updated_at": "2025-03-20T10:30:00Z"
      }
    ]
  },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 3
  }
}
```

### 5.2 Get Business Details

```http
GET /businesses/:business_id
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "FarmConnect Nigeria",
    "slug": "farmconnect-nigeria",
    "tagline": "Fresh farm produce, delivered to your door",
    "description": "Direct marketplace connecting farmers with urban consumers",
    "industry": "AgriTech",
    "sub_industry": "Farm-to-Table",
    "status": "active",
    "stage": "mvp",
    "country": "NG",
    "city": "Lagos",
    "founded_date": null,
    "registration_number": null,
    "legal_structure": "ltd",
    "website_url": "https://farmconnect-ng.venture.site",
    "custom_domain": null,
    "health_score": 72,
    "health_score_breakdown": {
      "compliance": 60,
      "digital_presence": 85,
      "documentation": 70,
      "financial_readiness": 65,
      "market_validation": 80
    },
    "logo": {
      "primary_url": "https://...",
      "variants": {
        "dark": "https://...",
        "light": "https://...",
        "icon": "https://..."
      }
    },
    "brand_colors": {
      "primary": "#2D5A3D",
      "secondary": "#F4A261",
      "accent": "#E76F51",
      "neutral": "#264653",
      "background": "#F8F9FA"
    },
    "generated_assets": {
      "business_plan": { "id": "uuid", "status": "completed", "url": "..." },
      "pitch_deck": { "id": "uuid", "status": "pending", "url": null },
      "website": { "id": "uuid", "status": "completed", "url": "..." },
      "branding_kit": { "id": "uuid", "status": "completed", "url": "..." }
    },
    "team": [
      {
        "user_id": "uuid",
        "role": "founder",
        "name": "John Doe",
        "email": "founder@example.com",
        "joined_at": "2025-03-20T10:00:00Z"
      }
    ],
    "created_at": "2025-03-20T10:00:00Z",
    "updated_at": "2025-03-20T10:30:00Z"
  }
}
```

### 5.3 Update Business

```http
PATCH /businesses/:business_id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "name": "FarmConnect Africa",
  "tagline": "Connecting farmers and consumers across Africa",
  "description": "...",
  "website_url": "https://farmconnect.africa"
}
```

### 5.4 Delete Business

```http
DELETE /businesses/:business_id
Authorization: Bearer <token>
```

### 5.5 Get Business Checklist

```http
GET /businesses/:business_id/checklist
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "overall_progress": 65,
    "categories": [
      {
        "name": "Legal & Compliance",
        "progress": 40,
        "items": [
          { "id": "1", "title": "Register business name", "completed": true, "priority": "high" },
          { "id": "2", "title": "Obtain TIN", "completed": false, "priority": "high" },
          { "id": "3", "title": "Register for VAT", "completed": false, "priority": "medium" }
        ]
      },
      {
        "name": "Digital Presence",
        "progress": 80,
        "items": [
          { "id": "4", "title": "Create website", "completed": true, "priority": "high" },
          { "id": "5", "title": "Set up social media", "completed": true, "priority": "medium" },
          { "id": "6", "title": "Connect custom domain", "completed": false, "priority": "low" }
        ]
      }
    ]
  }
}
```

### 5.6 Update Checklist Item

```http
PATCH /businesses/:business_id/checklist/:item_id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "completed": true,
  "notes": "Submitted application to CAC"
}
```

---

## 6. AI Generation Services

### 6.1 Generate Business Plan

```http
POST /businesses/:business_id/generate/business-plan
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "template": "standard",
  "sections": ["all"],
  "language": "en",
  "include_financials": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "job_id": "uuid",
    "status": "queued",
    "estimated_duration": "60s",
    "webhook_url": null
  }
}
```

### 6.2 Get Generation Job Status

```http
GET /generation-jobs/:job_id
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "type": "business_plan",
    "status": "completed",
    "progress": 100,
    "result": {
      "document_id": "uuid",
      "preview_url": "https://...",
      "download_url": "https://...",
      "format": "pdf"
    },
    "created_at": "2025-03-20T10:00:00Z",
    "started_at": "2025-03-20T10:00:05Z",
    "completed_at": "2025-03-20T10:01:02Z",
    "token_usage": 2500,
    "cost": 0.15
  }
}
```

### 6.3 Get Business Plan Content

```http
GET /businesses/:business_id/business-plan
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "business_id": "uuid",
    "version": 1,
    "content": {
      "executive_summary": "FarmConnect Nigeria is a digital marketplace...",
      "problem_statement": "Nigerian farmers lose up to 40% of their income...",
      "solution": "Our platform eliminates middlemen by connecting...",
      "market_analysis": {
        "tam": "$12 billion",
        "sam": "$800 million",
        "som": "$50 million",
        "growth_rate": "15% annually"
      },
      "business_model": {
        "revenue_streams": [...],
        "pricing": {...}
      },
      "competitive_analysis": [...],
      "marketing_strategy": {...},
      "financial_projections": {
        "year_1": {...},
        "year_2": {...},
        "year_3": {...}
      },
      "team": [...],
      "milestones": [...],
      "risk_analysis": [...]
    },
    "generated_at": "2025-03-20T10:01:02Z",
    "ai_model": "claude-3-5-sonnet-20241022"
  }
}
```

### 6.4 Generate Pitch Deck

```http
POST /businesses/:business_id/generate/pitch-deck
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "template": "vc_standard",
  "audience": "venture_capital",
  "slides_count": 12,
  "include_financials": true
}
```

### 6.5 Generate One-Pager

```http
POST /businesses/:business_id/generate/one-pager
Authorization: Bearer <token>
```

### 6.6 Regenerate Section

```http
POST /businesses/:business_id/generate/regenerate
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "document_type": "business_plan",
  "section": "executive_summary",
  "instructions": "Make it more concise and emphasize the social impact",
  "tone": "professional"
}
```

### 6.7 List Generation History

```http
GET /businesses/:business_id/generations
Authorization: Bearer <token>
```

---

## 7. Branding & Media

### 7.1 Generate Logo Options

```http
POST /businesses/:business_id/branding/generate-logos
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "style_preferences": ["modern", "minimalist", "nature"],
  "color_preferences": ["green", "orange"],
  "concept_keywords": ["farm", "connection", "fresh", "growth"],
  "variations_count": 4
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "job_id": "uuid",
    "status": "queued",
    "estimated_duration": "45s"
  }
}
```

### 7.2 Get Logo Options

```http
GET /businesses/:business_id/branding/logo-options
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "options": [
      {
        "id": "uuid",
        "url": "https://...",
        "thumbnail_url": "https://...",
        "style": "modern",
        "colors": ["#2D5A3D", "#F4A261"],
        "selected": false
      }
    ],
    "can_generate_more": true,
    "generations_remaining": 5
  }
}
```

### 7.3 Select Logo

```http
POST /businesses/:business_id/branding/select-logo
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "logo_id": "uuid",
  "generate_variants": true
}
```

### 7.4 Generate Brand Colors

```http
POST /businesses/:business_id/branding/generate-colors
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "base_color": "#2D5A3D",
  "mood": "trustworthy,fresh",
  "industry": "agritech"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "palette": {
      "primary": "#2D5A3D",
      "secondary": "#F4A261",
      "accent": "#E76F51",
      "neutral": "#264653",
      "background": "#F8F9FA",
      "text": "#1A1A1A",
      "success": "#2D5A3D",
      "warning": "#F4A261",
      "error": "#E76F51"
    },
    "ai_recommendation": "This palette conveys trust and growth..."
  }
}
```

### 7.5 Update Brand Colors

```http
PATCH /businesses/:business_id/branding/colors
Authorization: Bearer <token>
```

### 7.6 Get Brand Guidelines

```http
GET /businesses/:business_id/branding/guidelines
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "colors": {...},
    "typography": {
      "headings": "Inter",
      "body": "Inter",
      "accent": "Playfair Display"
    },
    "logo_usage": {
      "minimum_size": "32px",
      "clearspace": "16px",
      "dos": [...],
      "donts": [...]
    },
    "messaging": {
      "tone": "friendly,professional",
      "taglines": [...],
      "key_messages": [...]
    }
  }
}
```

### 7.7 Download Brand Kit

```http
GET /businesses/:business_id/branding/download
Authorization: Bearer <token>
```

**Query Parameters:**
- `format` (string): zip, pdf

### 7.8 Generate Marketing Materials

```http
POST /businesses/:business_id/branding/marketing-materials
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "materials": [
    { "type": "flyer", "size": "a4" },
    { "type": "social_banner", "platform": "linkedin" },
    { "type": "business_card", "orientation": "horizontal" }
  ]
}
```

---

## 8. Website Builder

### 8.1 Get Website Configuration

```http
GET /businesses/:business_id/website
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "business_id": "uuid",
    "subdomain": "farmconnect-nigeria",
    "custom_domain": null,
    "domain_status": "not_connected",
    "template": "startup-modern",
    "status": "published",
    "pages": [
      { "id": "home", "name": "Home", "slug": "/", "enabled": true },
      { "id": "about", "name": "About", "slug": "/about", "enabled": true },
      { "id": "contact", "name": "Contact", "slug": "/contact", "enabled": true }
    ],
    "seo": {
      "title": "FarmConnect Nigeria - Fresh Farm Produce Delivered",
      "description": "Connect directly with local farmers...",
      "keywords": ["farm", "fresh produce", "nigeria"],
      "og_image": "https://..."
    },
    "analytics": {
      "google_analytics_id": null,
      "facebook_pixel_id": null,
      "custom_scripts": []
    },
    "published_at": "2025-03-20T10:30:00Z",
    "last_modified": "2025-03-20T10:30:00Z",
    "public_url": "https://farmconnect-nigeria.venture.site"
  }
}
```

### 8.2 Update Website Configuration

```http
PATCH /businesses/:business_id/website
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "template": "startup-modern",
  "seo": {
    "title": "New Title",
    "description": "New description..."
  }
}
```

### 8.3 Get Page Content

```http
GET /businesses/:business_id/website/pages/:page_id
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "home",
    "name": "Home",
    "slug": "/",
    "sections": [
      {
        "id": "hero",
        "type": "hero",
        "content": {
          "headline": "Fresh from Farm to Your Door",
          "subheadline": "Connecting Nigerian farmers directly with urban consumers",
          "cta_text": "Start Shopping",
          "cta_link": "#products",
          "background_image": "https://..."
        }
      },
      {
        "id": "features",
        "type": "features",
        "content": {
          "title": "Why Choose FarmConnect?",
          "features": [
            { "icon": "truck", "title": "Fast Delivery", "description": "..." },
            { "icon": "leaf", "title": "100% Fresh", "description": "..." }
          ]
        }
      },
      {
        "id": "contact",
        "type": "contact",
        "content": {
          "title": "Get in Touch",
          "email": "hello@farmconnect.ng",
          "phone": "+234...",
          "address": "..."
        }
      }
    ]
  }
}
```

### 8.4 Update Page Content

```http
PATCH /businesses/:business_id/website/pages/:page_id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "sections": [
    {
      "id": "hero",
      "content": {
        "headline": "Updated Headline",
        "subheadline": "Updated description..."
      }
    }
  ]
}
```

### 8.5 Publish Website

```http
POST /businesses/:business_id/website/publish
Authorization: Bearer <token>
```

### 8.6 Unpublish Website

```http
POST /businesses/:business_id/website/unpublish
Authorization: Bearer <token>
```

### 8.7 Connect Custom Domain

```http
POST /businesses/:business_id/website/domain
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "domain": "farmconnect.ng"
}
```

### 8.8 Check Domain Status

```http
GET /businesses/:business_id/website/domain/status
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "domain": "farmconnect.ng",
    "status": "pending_dns",
    "dns_records": [
      { "type": "A", "name": "@", "value": "192.0.2.1", "required": true },
      { "type": "CNAME", "name": "www", "value": "farmconnect.venture.site", "required": false }
    ],
    "ssl_status": "pending",
    "message": "Please add the DNS records above to your domain registrar"
  }
}
```

### 8.9 List Available Templates

```http
GET /website/templates
Authorization: Bearer <token>
```

### 8.10 Preview Website

```http
GET /businesses/:business_id/website/preview
Authorization: Bearer <token>
```

---

## 9. Document Vault

### 9.1 List Documents

```http
GET /businesses/:business_id/documents
Authorization: Bearer <token>
```

**Query Parameters:**
- `folder_id` (string, optional)
- `type` (string, optional): pdf, image, contract, etc.
- `search` (string, optional)

**Response:**
```json
{
  "success": true,
  "data": {
    "documents": [
      {
        "id": "uuid",
        "name": "Business Plan v1.pdf",
        "type": "pdf",
        "size": 2450000,
        "folder_id": "uuid",
        "url": "https://...",
        "thumbnail_url": "https://...",
        "metadata": {
          "generated_by": "ai",
          "document_type": "business_plan",
          "version": 1
        },
        "created_at": "2025-03-20T10:01:00Z",
        "updated_at": "2025-03-20T10:01:00Z"
      }
    ],
    "folders": [
      {
        "id": "uuid",
        "name": "Legal Documents",
        "document_count": 5
      }
    ]
  }
}
```

### 9.2 Upload Document

```http
POST /businesses/:business_id/documents
Authorization: Bearer <token>
Content-Type: multipart/form-data
```

**Request Body:**
```
file: <binary_data>
folder_id: uuid (optional)
name: "Custom Name.pdf" (optional)
tags: ["contract", "legal"] (optional)
```

### 9.3 Get Document

```http
GET /businesses/:business_id/documents/:document_id
Authorization: Bearer <token>
```

### 9.4 Update Document

```http
PATCH /businesses/:business_id/documents/:document_id
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "name": "New Name.pdf",
  "folder_id": "uuid",
  "tags": ["updated", "tags"]
}
```

### 9.5 Delete Document

```http
DELETE /businesses/:business_id/documents/:document_id
Authorization: Bearer <token>
```

### 9.6 Create Folder

```http
POST /businesses/:business_id/documents/folders
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "name": "Legal Documents",
  "parent_id": null
}
```

### 9.7 Share Document

```http
POST /businesses/:business_id/documents/:document_id/share
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "expiry_days": 7,
  "password_protected": false,
  "allow_download": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "share_url": "https://app.venturemate.co/share/doc/uuid",
    "expires_at": "2025-03-27T10:00:00Z"
  }
}
```

### 9.8 Get Document Templates

```http
GET /document-templates
Authorization: Bearer <token>
```

---

## 10. Marketplace (Phase 2+)

### 10.1 List Services

```http
GET /marketplace/services
Authorization: Bearer <token>
```

**Query Parameters:**
- `category` (string): legal, accounting, marketing, design, development
- `country` (string): NG, ZA, KE, etc.
- `price_range` (string): budget, standard, premium
- `rating` (number): min rating

**Response:**
```json
{
  "success": true,
  "data": {
    "services": [
      {
        "id": "uuid",
        "title": "Company Registration - Nigeria",
        "provider": {
          "id": "uuid",
          "name": "LegalPro Nigeria",
          "rating": 4.8,
          "completed_projects": 150
        },
        "category": "legal",
        "description": "Complete CAC registration for your business...",
        "price": {
          "amount": 75000,
          "currency": "NGN",
          "pricing_model": "fixed"
        },
        "delivery_time": "7 days",
        "tags": ["cac", "registration", "limited_company"],
        "image_url": "https://..."
      }
    ]
  }
}
```

### 10.2 Get Service Details

```http
GET /marketplace/services/:service_id
Authorization: Bearer <token>
```

### 10.3 Request Service

```http
POST /marketplace/requests
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "service_id": "uuid",
  "business_id": "uuid",
  "requirements": "I need to register a limited company...",
  "attachments": ["uuid"]
}
```

### 10.4 List My Service Requests

```http
GET /marketplace/requests
Authorization: Bearer <token>
```

### 10.5 List Freelancers

```http
GET /marketplace/freelancers
Authorization: Bearer <token>
```

### 10.6 Post Job

```http
POST /marketplace/jobs
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "business_id": "uuid",
  "title": "Need logo refinement",
  "description": "Looking for a designer to refine our AI-generated logo...",
  "budget": {
    "min": 50000,
    "max": 100000,
    "currency": "NGN"
  },
  "skills_required": ["graphic_design", "branding"],
  "deadline": "2025-04-20"
}
```

---

## 11. Subscriptions & Billing

### 11.1 Get Subscription Plans

```http
GET /subscriptions/plans
```

**Response:**
```json
{
  "success": true,
  "data": {
    "plans": [
      {
        "id": "free",
        "name": "Free",
        "price": { "amount": 0, "currency": "USD" },
        "interval": "month",
        "features": [
          "1 business",
          "Basic AI generation",
          "Standard templates",
          "Community support"
        ],
        "limits": {
          "businesses": 1,
          "ai_generations_per_month": 10,
          "storage_gb": 1,
          "website_pages": 3
        }
      },
      {
        "id": "starter",
        "name": "Starter",
        "price": { "amount": 29, "currency": "USD" },
        "interval": "month",
        "features": [
          "3 businesses",
          "Advanced AI generation",
          "Premium templates",
          "Custom domain",
          "Priority support"
        ],
        "limits": {
          "businesses": 3,
          "ai_generations_per_month": 50,
          "storage_gb": 10,
          "website_pages": 10
        },
        "popular": true
      },
      {
        "id": "growth",
        "name": "Growth",
        "price": { "amount": 79, "currency": "USD" },
        "interval": "month",
        "features": [
          "Unlimited businesses",
          "Unlimited AI generation",
          "All templates",
          "Custom domain + SSL",
          "Analytics dashboard",
          "CRM tools",
          "24/7 priority support"
        ],
        "limits": {
          "businesses": -1,
          "ai_generations_per_month": -1,
          "storage_gb": 50,
          "website_pages": -1
        }
      }
    ]
  }
}
```

### 11.2 Get Current Subscription

```http
GET /subscriptions/current
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "plan": {
      "id": "starter",
      "name": "Starter"
    },
    "status": "active",
    "current_period_start": "2025-03-01T00:00:00Z",
    "current_period_end": "2025-04-01T00:00:00Z",
    "cancel_at_period_end": false,
    "payment_method": {
      "type": "card",
      "last4": "4242",
      "brand": "visa",
      "exp_month": 12,
      "exp_year": 2027
    },
    "usage": {
      "ai_generations_this_month": 23,
      "ai_generations_limit": 50,
      "storage_used_gb": 2.5,
      "storage_limit_gb": 10,
      "businesses_count": 2,
      "businesses_limit": 3
    }
  }
}
```

### 11.3 Subscribe to Plan

```http
POST /subscriptions
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "plan_id": "starter",
  "payment_method_id": "pm_stripe_id",
  "billing_interval": "month"
}
```

### 11.4 Update Subscription

```http
PATCH /subscriptions
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "plan_id": "growth"
}
```

### 11.5 Cancel Subscription

```http
POST /subscriptions/cancel
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "at_period_end": true
}
```

### 11.6 List Invoices

```http
GET /subscriptions/invoices
Authorization: Bearer <token>
```

### 11.7 Get Payment Methods

```http
GET /subscriptions/payment-methods
Authorization: Bearer <token>
```

### 11.8 Add Payment Method

```http
POST /subscriptions/payment-methods
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "payment_method_id": "pm_stripe_id"
}
```

---

## 12. Analytics & Health Score

### 12.1 Get Business Health Score

```http
GET /businesses/:business_id/health-score
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "overall_score": 72,
    "grade": "B+",
    "last_updated": "2025-03-20T10:00:00Z",
    "breakdown": {
      "compliance": {
        "score": 60,
        "weight": 20,
        "details": [
          { "factor": "business_registered", "score": 0, "max": 20 },
          { "factor": "tax_id_obtained", "score": 0, "max": 15 },
          { "factor": "required_licenses", "score": 10, "max": 10 },
          { "factor": "data_protection", "score": 15, "max": 15 }
        ]
      },
      "digital_presence": {
        "score": 85,
        "weight": 20,
        "details": [
          { "factor": "website_quality", "score": 25, "max": 25 },
          { "factor": "social_media_setup", "score": 20, "max": 20 },
          { "factor": "seo_optimization", "score": 20, "max": 20 },
          { "factor": "brand_consistency", "score": 20, "max": 20 }
        ]
      },
      "documentation": {
        "score": 70,
        "weight": 20,
        "details": [
          { "factor": "business_plan", "score": 20, "max": 20 },
          { "factor": "financial_projections", "score": 15, "max": 20 },
          { "factor": "pitch_deck", "score": 15, "max": 20 },
          { "factor": "legal_documents", "score": 20, "max": 20 }
        ]
      },
      "financial_readiness": {
        "score": 65,
        "weight": 20,
        "details": [
          { "factor": "business_bank_account", "score": 0, "max": 25 },
          { "factor": "payment_gateway", "score": 0, "max": 20 },
          { "factor": "accounting_system", "score": 20, "max": 20 },
          { "factor": "funding_strategy", "score": 25, "max": 25 }
        ]
      },
      "market_validation": {
        "score": 80,
        "weight": 20,
        "details": [
          { "factor": "customer_research", "score": 25, "max": 25 },
          { "factor": "competitive_analysis", "score": 20, "max": 20 },
          { "factor": "mvp_progress", "score": 15, "max": 20 },
          { "factor": "early_traction", "score": 20, "max": 20 }
        ]
      }
    },
    "recommendations": [
      {
        "id": "1",
        "priority": "high",
        "category": "compliance",
        "title": "Register your business with CAC",
        "description": "Formal registration is required for...",
        "action_url": "/businesses/uuid/registration",
        "potential_score_impact": 20
      },
      {
        "id": "2",
        "priority": "medium",
        "category": "finance",
        "title": "Open a business bank account",
        "description": "Separate personal and business finances...",
        "action_url": "/businesses/uuid/banking",
        "potential_score_impact": 15
      }
    ],
    "historical_scores": [
      { "date": "2025-03-15", "score": 65 },
      { "date": "2025-03-20", "score": 72 }
    ]
  }
}
```

### 12.2 Get Business Analytics

```http
GET /businesses/:business_id/analytics
Authorization: Bearer <token>
```

**Query Parameters:**
- `start_date` (ISO date)
- `end_date` (ISO date)
- `metrics` (array): views, unique_visitors, conversions

### 12.3 Get Website Analytics

```http
GET /businesses/:business_id/website/analytics
Authorization: Bearer <token>
```

**Response:**
```json
{
  "success": true,
  "data": {
    "period": "last_30_days",
    "summary": {
      "total_views": 1250,
      "unique_visitors": 890,
      "avg_session_duration": 145,
      "bounce_rate": 0.45
    },
    "top_pages": [
      { "page": "/", "views": 800 },
      { "page": "/about", "views": 200 }
    ],
    "traffic_sources": [
      { "source": "direct", "visitors": 400 },
      { "source": "social", "visitors": 300 },
      { "source": "search", "visitors": 150 }
    ],
    "devices": [
      { "device": "mobile", "percentage": 65 },
      { "device": "desktop", "percentage": 30 }
    ]
  }
}
```

---

## 13. Webhooks

### 13.1 Register Webhook

```http
POST /webhooks
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "url": "https://your-app.com/webhooks/venturemate",
  "events": [
    "business.created",
    "generation.completed",
    "subscription.updated"
  ],
  "secret": "your_webhook_secret",
  "active": true
}
```

### 13.2 List Webhooks

```http
GET /webhooks
Authorization: Bearer <token>
```

### 13.3 Delete Webhook

```http
DELETE /webhooks/:webhook_id
Authorization: Bearer <token>
```

### 13.4 Webhook Event Types

| Event | Description |
|-------|-------------|
| `user.created` | New user registered |
| `user.updated` | User profile updated |
| `business.created` | New business created |
| `business.updated` | Business details updated |
| `generation.started` | AI generation started |
| `generation.completed` | AI generation completed |
| `generation.failed` | AI generation failed |
| `subscription.created` | New subscription |
| `subscription.updated` | Subscription changed |
| `subscription.cancelled` | Subscription cancelled |
| `invoice.paid` | Invoice paid |
| `invoice.payment_failed` | Payment failed |

### 13.5 Webhook Payload Format

```json
{
  "id": "evt_uuid",
  "type": "generation.completed",
  "created_at": "2025-03-20T10:01:02Z",
  "data": {
    "job_id": "uuid",
    "business_id": "uuid",
    "user_id": "uuid",
    "type": "business_plan",
    "result": {
      "document_id": "uuid",
      "url": "https://..."
    }
  }
}
```

---

## 14. Error Handling

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_REQUEST` | 400 | Malformed request |
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |
| `SERVICE_UNAVAILABLE` | 503 | Temporary outage |

### Validation Error Details

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      {
        "field": "email",
        "code": "invalid_format",
        "message": "Must be a valid email address"
      },
      {
        "field": "password",
        "code": "too_short",
        "message": "Must be at least 8 characters"
      }
    ]
  }
}
```

### Rate Limiting

| Endpoint Type | Limit |
|---------------|-------|
| Authentication | 5 requests/minute |
| Standard API | 100 requests/minute |
| AI Generation | 10 requests/minute |

**Rate Limit Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1710931200
```

---

## 15. SDK & Client Libraries

### 15.1 Official SDKs

| Language | Package | Install |
|----------|---------|---------|
| JavaScript/TypeScript | `@venturemate/sdk` | `npm install @venturemate/sdk` |
| Python | `venturemate` | `pip install venturemate` |
| Rust | `venturemate-rs` | `cargo add venturemate-rs` |

### 15.2 JavaScript Example

```typescript
import { VentureMateClient } from '@venturemate/sdk';

const client = new VentureMateClient({
  apiKey: 'your_api_key',
  environment: 'production'
});

// Create a business
const business = await client.businesses.create({
  name: 'My Startup',
  industry: 'technology'
});

// Generate a business plan
const job = await client.ai.generateBusinessPlan(business.id, {
  template: 'standard'
});

// Poll for completion
const result = await client.jobs.waitForCompletion(job.id);
```

---

**API Version**: 1.0.0  
**Last Updated**: 2025-03-20  
**Contact**: api@venturemate.co
