# VentureMate - Deployment Guide

> Complete infrastructure and deployment documentation

## 📋 Table of Contents

1. [Infrastructure Overview](#1-infrastructure-overview)
2. [Local Development](#2-local-development)
3. [Staging Deployment](#3-staging-deployment)
4. [Production Deployment](#4-production-deployment)
5. [Database Management](#5-database-management)
6. [Monitoring & Alerting](#6-monitoring--alerting)
7. [Security Checklist](#7-security-checklist)
8. [Disaster Recovery](#8-disaster-recovery)

---

## 1. Infrastructure Overview

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PRODUCTION ARCHITECTURE                         │
└─────────────────────────────────────────────────────────────────────────────┘

                                    ┌──────────┐
                                    │   User   │
                                    └────┬─────┘
                                         │
                              ┌──────────┴──────────┐
                              │   CloudFlare CDN    │
                              │  (SSL, DDoS, Cache) │
                              └──────────┬──────────┘
                                         │
                              ┌──────────┴──────────┐
                              │   AWS ALB / NLB     │
                              │   (Load Balancer)   │
                              └──────────┬──────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
            ┌───────▼──────┐    ┌───────▼──────┐    ┌───────▼──────┐
            │  API Server  │    │  API Server  │    │  API Server  │
            │    (Rust)    │    │    (Rust)    │    │    (Rust)    │
            │   ECS Task   │    │   ECS Task   │    │   ECS Task   │
            └───────┬──────┘    └───────┬──────┘    └───────┬──────┘
                    │                    │                    │
                    └────────────────────┼────────────────────┘
                                         │
                              ┌──────────┴──────────┐
                              │   ElastiCache       │
                              │   (Redis Cluster)   │
                              └─────────────────────┘
                                         │
                    ┌────────────────────┼────────────────────┐
                    │                    │                    │
            ┌───────▼──────┐    ┌───────▼──────┐    ┌───────▼──────┐
            │   RDS        │    │   S3         │    │   SQS        │
            │ PostgreSQL   │    │   (Files)    │    │  (Queue)     │
            │  (Multi-AZ)  │    │              │    │              │
            └──────────────┘    └──────────────┘    └──────────────┘
```

### Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **DNS/CDN** | CloudFlare | DNS, SSL, DDoS protection, caching |
| **Load Balancer** | AWS ALB | Traffic distribution, health checks |
| **Compute** | AWS ECS Fargate | Container orchestration |
| **Database** | AWS RDS PostgreSQL | Primary data storage |
| **Cache** | AWS ElastiCache Redis | Sessions, rate limiting, cache |
| **Storage** | AWS S3 | File uploads, static assets |
| **Queue** | AWS SQS | Background job processing |
| **Monitoring** | CloudWatch + Grafana | Metrics, logs, alerts |

---

## 2. Local Development

### Docker Compose Setup

**File: `docker-compose.yml`**

```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgres://postgres:postgres@postgres:5432/venturemate
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=dev-secret-key
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
    volumes:
      - ./src:/app/src
    depends_on:
      - postgres
      - redis
    command: cargo watch -x run

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: venturemate
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  mailhog:
    image: mailhog/mailhog
    ports:
      - "1025:1025"  # SMTP
      - "8025:8025"  # Web UI

volumes:
  postgres_data:
  redis_data:
```

**Run Locally:**
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f api

# Run migrations
docker-compose exec api cargo sqlx migrate run

# Stop everything
docker-compose down

# Reset data
docker-compose down -v
```

---

## 3. Staging Deployment

### AWS Setup

#### Step 1: Create ECR Repository

```bash
# Create ECR repository
aws ecr create-repository --repository-name venturemate-api --region us-east-1

# Login to ECR
aws ecr get-login-password | docker login --username AWS --password-stdin <account>.dkr.ecr.us-east-1.amazonaws.com
```

#### Step 2: Build and Push Docker Image

```bash
# Build image
docker build -t venturemate-api .

# Tag for ECR
docker tag venturemate-api:latest <account>.dkr.ecr.us-east-1.amazonaws.com/venturemate-api:latest

# Push
docker push <account>.dkr.ecr.us-east-1.amazonaws.com/venturemate-api:latest
```

**Dockerfile:**

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/bcknd /app/bcknd

# Create non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8080

CMD ["./bcknd"]
```

#### Step 3: Terraform Infrastructure

**File: `infra/staging/main.tf`**

```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC
module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "5.0.0"

  name = "venturemate-staging"
  cidr = "10.0.0.0/16"

  azs             = ["us-east-1a", "us-east-1b"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24"]

  enable_nat_gateway = true
  single_nat_gateway = true
}

# ECS Cluster
resource "aws_ecs_cluster" "main" {
  name = "venturemate-staging"
}

# RDS PostgreSQL
resource "aws_db_instance" "postgres" {
  identifier        = "venturemate-staging"
  engine           = "postgres"
  engine_version   = "15.4"
  instance_class   = "db.t3.micro"
  allocated_storage = 20

  db_name  = "venturemate"
  username = "postgres"
  password = var.db_password

  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name

  backup_retention_period = 7
  skip_final_snapshot    = true
}

# ElastiCache Redis
resource "aws_elasticache_cluster" "redis" {
  cluster_id           = "venturemate-staging"
  engine              = "redis"
  node_type           = "cache.t3.micro"
  num_cache_nodes     = 1
  parameter_group_name = "default.redis7"
  port                = 6379
  security_group_ids  = [aws_security_group.redis.id]
  subnet_group_name   = aws_elasticache_subnet_group.main.name
}

# ECS Service
resource "aws_ecs_service" "api" {
  name            = "api"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.api.arn
  desired_count   = 2
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = module.vpc.private_subnets
    security_groups  = [aws_security_group.ecs.id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.api.arn
    container_name   = "api"
    container_port   = 8080
  }
}

# Application Load Balancer
resource "aws_lb" "main" {
  name               = "venturemate-staging"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets           = module.vpc.public_subnets
}

# Output
output "load_balancer_dns" {
  value = aws_lb.main.dns_name
}
```

**Deploy:**
```bash
cd infra/staging

# Initialize Terraform
terraform init

# Plan changes
terraform plan

# Apply
terraform apply

# Get outputs
terraform output
```

---

## 4. Production Deployment

### Production Checklist

#### Pre-Deployment

- [ ] All tests passing
- [ ] Security audit complete
- [ ] Performance benchmarks met
- [ ] Documentation updated
- [ ] Database migrations tested
- [ ] Rollback plan documented

#### Deployment Steps

```bash
#!/bin/bash
# deploy-production.sh

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: ./deploy-production.sh <version>"
    exit 1
fi

echo "🚀 Deploying version $VERSION to production..."

# 1. Build and tag
docker build -t venturemate-api:$VERSION .
docker tag venturemate-api:$VERSION $ECR_URL/venturemate-api:$VERSION
docker tag venturemate-api:$VERSION $ECR_URL/venturemate-api:latest

# 2. Push to ECR
docker push $ECR_URL/venturemate-api:$VERSION
docker push $ECR_URL/venturemate-api:latest

# 3. Database migrations (run from CI/CD)
echo "Running database migrations..."
cargo sqlx migrate run --database-url $PROD_DATABASE_URL

# 4. Update ECS service
echo "Updating ECS service..."
aws ecs update-service \
    --cluster venturemate-production \
    --service api \
    --force-new-deployment

# 5. Wait for deployment
echo "Waiting for deployment to complete..."
aws ecs wait services-stable \
    --cluster venturemate-production \
    --services api

# 6. Health check
echo "Running health checks..."
curl -f https://api.venturemate.co/health || exit 1

echo "✅ Deployment complete!"
```

### Blue/Green Deployment

```hcl
# infra/production/bluegreen.tf

resource "aws_codedeploy_app" "api" {
  name = "venturemate-api"
  compute_platform = "ECS"
}

resource "aws_codedeploy_deployment_group" "api" {
  app_name              = aws_codedeploy_app.api.name
  deployment_group_name = "api"
  service_role_arn      = aws_iam_role.codedeploy.arn

  deployment_config_name = "CodeDeployDefault.ECSAllAtOnce"

  ecs_service {
    cluster_name = aws_ecs_cluster.main.name
    service_name = aws_ecs_service.api.name
  }

  load_balancer_info {
    target_group_pair_info {
      prod_traffic_route {
        listener_arns = [aws_lb_listener.https.arn]
      }

      target_group {
        name = aws_lb_target_group.blue.name
      }

      target_group {
        name = aws_lb_target_group.green.name
      }
    }
  }
}
```

---

## 5. Database Management

### Backup Strategy

```bash
#!/bin/bash
# backup-database.sh

DB_INSTANCE="venturemate-production"
S3_BUCKET="venturemate-db-backups"
DATE=$(date +%Y%m%d_%H%M%S)

# Create snapshot
aws rds create-db-snapshot \
    --db-instance-identifier $DB_INSTANCE \
    --db-snapshot-identifier "venturemate-${DATE}"

# Export to S3 (for long-term storage)
aws rds start-export-task \
    --export-task-identifier "venturemate-export-${DATE}" \
    --source-arn "arn:aws:rds:us-east-1:account:snapshot:venturemate-${DATE}" \
    --s3-bucket-name $S3_BUCKET \
    --iam-role-arn "arn:aws:iam::account:role/RDSExportRole" \
    --kms-key-id "arn:aws:kms:us-east-1:account:key/key-id"
```

### Migration Best Practices

```bash
# Run migrations in transaction
# migrations should be idempotent

# Check migration status
cargo sqlx migrate info --database-url $DATABASE_URL

# Run pending migrations
cargo sqlx migrate run --database-url $DATABASE_URL

# Revert last migration (emergency only)
cargo sqlx migrate revert --database-url $DATABASE_URL
```

---

## 6. Monitoring & Alerting

### CloudWatch Dashboard

```json
{
  "widgets": [
    {
      "type": "metric",
      "properties": {
        "title": "API Requests",
        "metrics": [
          ["AWS/ApplicationELB", "RequestCount", "LoadBalancer", "${alb_arn_suffix}"]
        ],
        "period": 60,
        "stat": "Sum"
      }
    },
    {
      "type": "metric",
      "properties": {
        "title": "Response Time",
        "metrics": [
          ["AWS/ApplicationELB", "TargetResponseTime", "LoadBalancer", "${alb_arn_suffix}"]
        ],
        "period": 60,
        "stat": "p99"
      }
    },
    {
      "type": "metric",
      "properties": {
        "title": "Error Rate",
        "metrics": [
          ["AWS/ApplicationELB", "HTTPCode_Target_5XX_Count", "LoadBalancer", "${alb_arn_suffix}"],
          [".", "HTTPCode_Target_4XX_Count", ".", "."]
        ],
        "period": 60,
        "stat": "Sum"
      }
    }
  ]
}
```

### Alerting Rules

```yaml
# cloudwatch-alarms.yml

Resources:
  HighErrorRateAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: venturemate-high-error-rate
      MetricName: HTTPCode_Target_5XX_Count
      Namespace: AWS/ApplicationELB
      Statistic: Sum
      Period: 300
      EvaluationPeriods: 2
      Threshold: 10
      ComparisonOperator: GreaterThanThreshold
      AlarmActions:
        - !Ref SNSTopic

  HighResponseTimeAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: venturemate-high-response-time
      MetricName: TargetResponseTime
      Namespace: AWS/ApplicationELB
      ExtendedStatistic: p99
      Period: 300
      EvaluationPeriods: 2
      Threshold: 1.0
      ComparisonOperator: GreaterThanThreshold
      AlarmActions:
        - !Ref SNSTopic

  DatabaseCPUAlarm:
    Type: AWS::CloudWatch::Alarm
    Properties:
      AlarmName: venturemate-db-high-cpu
      MetricName: CPUUtilization
      Namespace: AWS/RDS
      Dimensions:
        - Name: DBInstanceIdentifier
          Value: venturemate-production
      Statistic: Average
      Period: 300
      EvaluationPeriods: 2
      Threshold: 80
      ComparisonOperator: GreaterThanThreshold
      AlarmActions:
        - !Ref SNSTopic
```

### Structured Logging

```rust
// src/utils/logging.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true),
        )
        .init();
}

// Usage in handlers
#[tracing::instrument(skip(req), fields(user_id = %user_id))]
async fn create_business(
    req: HttpRequest,
    // ...
) -> HttpResponse {
    tracing::info!("Creating new business");
    
    match service.create_business().await {
        Ok(business) => {
            tracing::info!(business_id = %business.id, "Business created successfully");
            HttpResponse::Created().json(business)
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to create business");
            e.into_response()
        }
    }
}
```

---

## 7. Security Checklist

### Pre-Launch Security

- [ ] SSL/TLS certificates valid and auto-renewing
- [ ] Security headers configured (HSTS, CSP, etc.)
- [ ] CORS properly configured
- [ ] Rate limiting enabled
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention (parameterized queries)
- [ ] XSS protection
- [ ] Secrets management (AWS Secrets Manager)
- [ ] Network security groups configured
- [ ] WAF rules active
- [ ] Penetration test completed
- [ ] GDPR/privacy compliance verified

### Security Headers

```rust
// src/middleware/security.rs
use actix_web::http::header;

pub fn security_headers() -> impl Transform<S, Service = impl Service> {
    |req: ServiceRequest| {
        let res = req.into_response(...);
        
        res.headers_mut().insert(
            header::STRICT_TRANSPORT_SECURITY,
            header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
        
        res.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            header::HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"
            ),
        );
        
        res.headers_mut().insert(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("DENY"),
        );
        
        res.headers_mut().insert(
            header::X_CONTENT_TYPE_OPTIONS,
            header::HeaderValue::from_static("nosniff"),
        );
        
        res
    }
}
```

---

## 8. Disaster Recovery

### Recovery Objectives

| Metric | Target | Description |
|--------|--------|-------------|
| RPO | 1 hour | Maximum data loss acceptable |
| RTO | 4 hours | Maximum downtime acceptable |

### Runbook: Database Recovery

```bash
#!/bin/bash
# disaster-recovery.sh

# Scenario: Database corruption

# 1. Identify last good snapshot
SNAPSHOT_ID=$(aws rds describe-db-snapshots \
    --db-instance-identifier venturemate-production \
    --query 'DBSnapshots[?Status==`available`]|[0].DBSnapshotIdentifier' \
    --output text)

echo "Restoring from snapshot: $SNAPSHOT_ID"

# 2. Create new instance from snapshot
aws rds restore-db-instance-from-db-snapshot \
    --db-instance-identifier venturemate-production-recovery \
    --db-snapshot-identifier $SNAPSHOT_ID \
    --db-instance-class db.r6g.xlarge

# 3. Wait for restoration
aws rds wait db-instance-available \
    --db-instance-identifier venturemate-production-recovery

# 4. Update application to use recovered database
# (Requires manual update of DATABASE_URL or DNS change)

echo "Recovery complete. Update application configuration to point to venturemate-production-recovery"
```

### Incident Response

```
INCIDENT SEVERITY LEVELS

SEV-1 (Critical): Complete service outage
- Response: Immediate (15 min)
- Actions: Page on-call engineer, executive notification
- Examples: Database down, all APIs returning 500

SEV-2 (High): Significant impact
- Response: Within 1 hour
- Actions: Slack alert, team lead notification
- Examples: Payment processing down, major feature broken

SEV-3 (Medium): Partial impact
- Response: Within 4 hours
- Actions: Ticket creation
- Examples: Single region slow, minor feature degraded

SEV-4 (Low): Minimal impact
- Response: Next business day
- Actions: Backlog ticket
- Examples: Cosmetic issues, minor performance degradation
```

---

## Quick Commands Reference

```bash
# View ECS logs
aws logs tail /ecs/venturemate-api --follow

# Scale ECS service
aws ecs update-service --cluster venturemate-production --service api --desired-count 4

# Force new deployment
aws ecs update-service --cluster venturemate-production --service api --force-new-deployment

# Check service status
aws ecs describe-services --cluster venturemate-production --services api

# Database connection
psql $DATABASE_URL

# Redis connection
redis-cli -h $REDIS_HOST -p 6379

# SSH to ECS task (using ECS Exec)
aws ecs execute-command \
    --cluster venturemate-production \
    --task <task-id> \
    --container api \
    --interactive \
    --command "/bin/sh"
```

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-03-20  
**Infrastructure Status**: Ready for deployment
