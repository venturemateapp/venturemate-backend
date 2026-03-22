# VentureMate Email Service

This document describes the email service implementation for VentureMate.

## Overview

The email service provides transactional email capabilities including:
- Password reset emails
- Welcome emails for new users
- Login alert notifications
- Email verification
- Password change confirmations

## Architecture

### Components

1. **EmailService** (`src/services/email_service.rs`)
   - Core email handling with SMTP integration
   - Template rendering with VentureMate branding
   - Email logging for audit trails

2. **AuthService Integration** (`src/services/auth_service.rs`)
   - Triggers emails during auth flows
   - Login alerts on successful authentication

3. **Database Tables**
   - `email_logs` - Tracks all sent emails
   - `password_reset_tokens` - Secure token storage for password resets

### Email Templates

All emails use a consistent dark-themed template with VentureMate branding:

- **Header**: VentureMate logo with gradient
- **Body**: Customizable content based on email type
- **CTA Button**: Prominent action buttons for user flows
- **Footer**: Links to website, settings, and privacy policy

### Supported Templates

| Template | Purpose | Trigger |
|----------|---------|---------|
| `welcome` | New user onboarding | User registration |
| `password_reset` | Password reset link | Forgot password request |
| `password_changed` | Password change confirmation | After successful reset |
| `login_alert` | New device/login notification | Successful login |
| `email_verification` | Verify email address | Registration/OAuth |
| `email_verified` | Email confirmed | After verification |

## Configuration

### Environment Variables

```bash
# SMTP Configuration (Gmail recommended)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
SMTP_FROM_EMAIL=noreply@venturemate.app
SMTP_FROM_NAME=VentureMate

# Frontend URL for email links
FRONTEND_URL=http://localhost:3000
```

### Gmail App Password Setup

1. Go to [Google Account Settings](https://myaccount.google.com)
2. Enable 2-Step Verification
3. Go to Security > App Passwords
4. Generate an app password for "Mail"
5. Use this password in `SMTP_PASS` (not your regular password)

## API Endpoints

### Password Reset Flow

```
POST /api/v1/auth/password-reset-request
Body: { "email": "user@example.com" }

POST /api/v1/auth/password-reset
Body: { "token": "reset-token", "new_password": "newpass123" }
```

### Email Verification

```
GET /api/v1/auth/verify-email?token=verification-token

POST /api/v1/auth/resend-verification
Headers: Authorization: Bearer <token>
```

## Frontend Integration

### Password Reset Pages

- **Forgot Password**: `/auth/forgot-password`
  - Form to request reset link
  - Shows confirmation message

- **Reset Password**: `/auth/reset-password?token=<token>`
  - Validates token from URL
  - Form for new password
  - Redirects to signin on success

### API Client Methods

```typescript
// Request password reset
await authApi.forgotPassword(email);

// Reset password with token
await authApi.resetPassword(token, newPassword);
```

## Security Features

### Password Reset Tokens

- **Expiration**: 24 hours
- **Storage**: Bcrypt hashed in database
- **One-time use**: Marked as used after successful reset
- **Session revocation**: All sessions invalidated after password change

### Login Alerts

- **IP Tracking**: Records login IP address
- **User Agent**: Captures device/browser info
- **Timestamp**: Exact login time

### Email Logging

All emails are logged for:
- Compliance auditing
- Delivery tracking
- Debugging purposes

## Development Mode

When SMTP is not configured, emails are:
- Logged to console instead of sent
- Stored in `email_logs` with status "logged"
- Still fully functional for testing

## Database Schema

### email_logs

```sql
CREATE TABLE email_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    business_id UUID REFERENCES businesses(id) ON DELETE SET NULL,
    email_type VARCHAR(50) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    subject VARCHAR(500) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### password_reset_tokens

```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET,
    user_agent TEXT
);
```

## Testing

### Manual Testing

1. **Password Reset Flow**:
   ```bash
   # Request reset
   curl -X POST http://localhost:8080/api/v1/auth/password-reset-request \
     -H "Content-Type: application/json" \
     -d '{"email": "test@example.com"}'
   
   # Check logs for token
   docker logs venturemate-backend | grep "Password reset token"
   
   # Reset password
   curl -X POST http://localhost:8080/api/v1/auth/password-reset \
     -H "Content-Type: application/json" \
     -d '{"token": "token-from-logs", "new_password": "newpass123"}'
   ```

2. **Login Alert**:
   - Sign in to trigger email
   - Check email_logs table for entry

## Troubleshooting

### Emails Not Sending

1. Check SMTP configuration in `.env`
2. Verify Gmail app password (not regular password)
3. Check server logs for SMTP errors
4. Ensure `FRONTEND_URL` is set correctly

### Token Expired

- Reset tokens expire after 24 hours
- Request a new reset link

### Email in Spam

- Verify SPF/DKIM records for custom domain
- Use Gmail SMTP for better deliverability
- Check email content for spam triggers
