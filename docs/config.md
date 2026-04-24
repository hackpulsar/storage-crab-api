# Configuration

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://admin:admin@db:5432/crab_storage` |
| `REDIS_URL` | Redis connection string | `redis://redis:6379` |
| `DATABASE_USER` | PostgreSQL username | `admin` |
| `DATABASE_PASSWORD` | PostgreSQL password | `admin` |
| `DATABASE_NAME` | PostgreSQL database name | `crab_storage` |
| `FILES_STORAGE_PATH` | Path for stored files | `/app/files_storage` |
| `TMP_FILES_STORAGE` | Path for temporary files | `/app/files_storage/tmp` |
| `RUST_LOG` | Logging level | `debug` |

## Prerequisites

- Rust (latest stable)
- Docker & Docker Compose
- PostgreSQL 16
- Redis 7

## TLS Certificates

Place `cert.pem` and `key.pem` in the project root for HTTPS support.