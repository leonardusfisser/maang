# 🧱 MAANG FRAMEWORK — ARCHITECTURE

## Overview
The MAANG Framework is a **4-layer Rust architecture** designed for multi-tenant SaaS apps with per-tenant SurrealDB isolation.

web → application → domain → infrastructure → core


Each layer builds downward dependencies only. No circular calls.

## Layers
- **CORE** — config, errors, observability, utils
- **DOMAIN** — tenants, users, products, services, media
- **INFRASTRUCTURE** — surrealdb, cache, email, storage, stripe, metrics
- **APPLICATION** — api, web, cli
- **APPS** — tenant-specific binaries (e.g., lillpepe)

## Principles
- No `unwrap`, `expect`, or panics
- All errors are typed (`AppError`, `DomainError`, `InfraError`)
- Tenant isolation required in every repo method
- Observability via `tracing` spans (`request_id`, `tenant_id`)
- Workspace lints deny warnings and panics

## Data Flow
`HTTP Request → Actix Web → Application Service → Domain Logic → SurrealDB`

## Build Profile
```toml
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
strip = true
panic = "abort"


