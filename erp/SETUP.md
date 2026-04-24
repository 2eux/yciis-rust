# Secure School ERP - Setup & Deployment

## Prerequisites

- **Runtime**: Rust 1.75+, Node.js 18+
- **Database**: PostgreSQL 15+ (Supabase managed or self-hosted)
- **Cache**: Redis 7+
- **Container**: Docker & Docker Compose
- **Deployment**: Coolify (recommended) or any Docker host

---

## Quick Start (Development)

### 1. Clone & Setup
```bash
# Clone the repository
git clone https://github.com/yourusername/secure-school-erp.git
cd secure-school-erp

# Copy environment template
cp .env.example .env
# Edit .env with your values
```

### 2. Backend
```bash
cd backend
cargo build
cargo run
# Server starts on http://localhost:3000
```

### 3. Frontend
```bash
cd frontend
npm install
npm run dev
# Opens on http://localhost:5173
```

---

## Docker Deployment (Local)

```bash
cd infra
docker-compose up -d --build

# Check status
docker-compose ps
docker-compose logs -f api
```

---

## Production Deployment (Coolify)

### 1. Build & Push Images
```bash
# Backend
cd backend
docker build -t ghcr.io/yourusername/school-erp-api:latest .
docker push ghcr.io/yourusername/school-erp-api:latest

# Frontend
cd ../frontend
docker build -t ghcr.io/yourusername/school-erp-frontend:latest .
docker push ghcr.io/yourusername/school-erp-frontend:latest
```

### 2. Configure Secrets in Coolify
Add these secrets in Coolify UI:

| Secret | Example | Description |
|--------|---------|------------|
| `database_url` | `postgresql://user:pass@host:5432/db` | PostgreSQL |
| `redis_url` | `redis://:pass@host:6379` | Redis |
| `jwt_secret` | `openssl rand -base64 32` | JWT signing key |
| `supabase_url` | `https://xyz.supabase.co` | Supabase URL |
| `supabase_anon_key` | `eyJhbG...` | Supabase anon key |
| `supabase_service_key` | `eyJhbG...` | Supabase service key |

### 3. Deploy
1. Create new application in Coolify
2. Upload or reference `coolify-api.json` as definition
3. Add all secrets
4. Enable PostgreSQL and Redis resources
5. Deploy

---

## Database Setup

Run this in Supabase SQL editor (`docs/schema.sql`):

```sql
-- Creates all required tables
-- Tables: users, roles, permissions, user_roles, user_scopes, sessions, audit_logs
-- Default roles: admin, yayasan, finance, hr, teacher, student, parent, donor, public
```

---

## API Endpoints

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/v1/auth/register` | Register user |
| POST | `/api/v1/auth/login` | Login |
| POST | `/api/v1/auth/refresh` | Refresh token |
| POST | `/api/v1/auth/logout` | Logout |
| POST | `/api/v1/auth/enable-2fa` | Enable 2FA |
| POST | `/api/v1/auth/verify-2fa` | Verify 2FA |
| GET | `/api/v1/auth/me` | Current user |

### RBAC
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/rbac/roles` | List roles |
| GET | `/api/v1/rbac/permissions` | List permissions |
| POST | `/api/v1/rbac/check` | Check access |

### Audit
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/audit/logs` | Get logs |
| GET | `/api/v1/audit/stats` | Get statistics |
| POST | `/api/v1/audit/query` | Query logs |

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `REDIS_URL` | Yes | Redis connection string |
| `JWT_SECRET` | Yes | JWT signing key (min 32 chars) |
| `SUPABASE_URL` | No | Supabase URL |
| `SUPABASE_ANON_KEY` | No | Supabase anon key |
| `SUPABASE_SERVICE_KEY` | No | Supabase service key |
| `PORT` | No | Server port (default: 3000) |
| `RUST_LOG` | No | Log level (default: info) |

---

## Security Features

- [x] JWT with short expiry (15 min)
- [x] Refresh token rotation
- [x] TOTP 2FA support
- [x] Rate limiting (5 attempts then lockout)
- [x] RBAC with scope-based authorization
- [x] Full audit logging
- [x] Role-based access control (9 roles)
- [x] Permission system (28+ permissions)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Client (Web/Chat)                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
               ┌──────▼──────┐
               │  Cloudflare  │  (WAF + SSL + Rate Limiting)
               └──────┬──────┘
                      │
               ┌──────▼──────┐
               │   Coolify   │  (Deployment + Health)
               └──────┬──────┘
                      │
        ┌───────────────┼───────────────┐
        │               │               │
   ┌────▼────┐    ┌────▼────┐    ┌──▼────┐
   │  API   │    │  Redis │    │   DB  │
   │ (Rust) │───▶│ Cache  │───▶│Postgres│
   └────┬────┘    └────────┘    └───┬───┘
        │                         │
        └───────────┬───────────────┘
                    │
              ┌─────▼─────┐
              │  Backup   │  (rclone → Drive)
              └──────────┘
```

---

## Troubleshooting

### Build fails
```bash
# Install build dependencies
cargo build --features production
```

### Connection refused
```bash
# Check services are running
docker-compose ps
docker-compose logs redis
```

### Database errors
```bash
# Verify DATABASE_URL format
# postgresql://user:password@host:port/database
```

### JWT errors
```bash
# Regenerate JWT secret
openssl rand -base64 32
```

---

## License

Proprietary - All rights reserved