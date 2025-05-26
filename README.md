# 🏥 Hospital Management System (HMS) – Regional Focused

A modern, scalable, and secure hospital management system tailored for Indonesian and Southeast Asian healthcare providers. Built using Rust and an advanced tech stack, this project is focused on reliability, modularity, and performance in real-world hospital operations.

---

## ✨ Features

- 🚀 High-performance backend built with Rust
- 🧠 ORM integration using SeaORM
- 🔒 Authentication & role-based access (planned)
- 📅 Appointment scheduling system (planned)
- 🗂️ Patient records management (planned)
- ☁️ File upload to AWS S3 (e.g., medical scans, test results)
- 📊 Redis-powered queueing & real-time modules
- 🐘 PostgreSQL for relational data modeling
- 🐳 Docker-ready for easy deployment

---

## 🔧 Tech Stack

| Layer        | Technology           |
|--------------|----------------------|
| Language     | Rust                 |
| ORM          | SeaORM               |
| Cache & Queue| Redis                |
| Database     | PostgreSQL           |
| Storage      | AWS S3               |
| Container    | Docker               |
| Migration    | SeaORM CLI           |

---

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Docker](https://www.docker.com/) & Docker Compose
- PostgreSQL (local or via Docker)
- Redis (local or via Docker)

### Clone

```bash
git clone https://github.com/Xenn-00/hospital_management_system.git
cd hospital_management_system
```

### Setup `application.yaml`

```yaml
name: HMS
version: 1.0.0
description: Hospital Management System

database:
  url: postgres://YOUR_USER:YOUR_PASSWORD@localhost:6432/hms_db
  driver: postgres
  direct_url: postgres://YOUR_USER:YOUR_PASSWORD@localhost:5432/hms_db

redis:
  official_url: redis://default:YOUR_REDIS_PASSWORD@YOUR_REDIS_HOST:PORT
  upstash_redis_url: rediss://default:YOUR_REDIS_PASSWORD@YOUR_UPSTASH_DOMAIN:6379

s3:
  s3_url: http://localhost:9000
  s3_region: ap-southeast-3
  s3_access_key: your-access-key
  s3_secret_key: your-secret-key

app:
  host: "127.0.0.1"
  port: 8080
  cors:
    allow_origin: "*"
    allow_methods: "GET, POST, PUT, DELETE, OPTIONS"
    allow_headers: "Content-Type, Authorization"
```

### run

```bash
cargo run
```

## 📄 License
This project is licensed under the [MIT LICENSE](LICENSE) 
