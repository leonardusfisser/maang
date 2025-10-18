# 🧠 MAANG FRAMEWORK

**MAANG Framework** is a production-grade **Rust monorepo** containing a shared framework and independent SaaS apps.  
It provides a common foundation (core/domain/infra/application) for multiple white-label Rust products — each app builds its **own binary** and ships fully self-contained.

---

## 🏗️ Monorepo Architecture

The MAANG structure separates shared framework crates from app binaries.


### Layers

| Layer | Description |
|-------|--------------|
| **Core** | Shared foundation (`config`, `errors`, `observability`, `utils`) — panic-free. |
| **Domain** | Business logic per module (`users`, `tenants`, `products`, etc.). |
| **Infrastructure** | Database and integrations (SurrealDB, Postgres, Mongo, Stripe, cache, email). |
| **Application** | Orchestration and interfaces (web, api, cli). |
| **Apps** | Independent binaries, each targeting different domains. |

---

## 🧩 Core Principles

- **No panics:** CI forbids `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`.
- **Typed errors:** Distinct `AppError`, `DomainError`, and `InfraError`.
- **Tenant isolation:** Every repository and service enforces `tenant_id`.
- **Observability:** `tracing` spans carry `request_id` + `tenant_id`.
- **Security-first:** Nginx + Cloudflare TLS proxy, no Docker or YAML.
- **Performance:** Compiled HTML/CSS/JS/static assets in binaries.
- **Simplicity:** Minimal dependencies, binary-only distribution.

---

## 🧰 Technology Stack

| Category | Tech |
|-----------|------|
| Language | Rust (edition 2024, panic=abort, LTO=thin) |
| Runtime | Tokio |
| Web | Actix Web + Askama (SSR-first, HTMX optional) |
| Database | SurrealDB (LillPepe), PostgreSQL (Radar), PostgreSQL + MongoDB (BigPepe) |
| Observability | `tracing` |
| Payments | Stripe |
| OS | Parrot OS (dev) / Debian (server) |
| CLI | Fish shell |

---

## ⚙️ Build & Run (Fish Shell)

# Clone repository
git clone https://github.com/leonardusfisser/maangframe.git
cd maangframe

# Build full workspace (release)
cargo build --release

# Run an example app
cargo run -p lillpepe

apps/ → independent binaries (LillPepe, Radar, Streaming, BigPepe)
crates/ → shared framework (core, domain, infra, application)
ops/ → operational binaries (backup, archive, release, provision)
assets/ → static assets (html, css, js, media)
docs/ → documentation (architecture, deploy, security, etc.)


### Layers

| Layer | Description |
|-------|--------------|
| **Core** | Shared foundation (`config`, `errors`, `observability`, `utils`) — panic-free. |
| **Domain** | Business logic per module (`users`, `tenants`, `products`, etc.). |
| **Infrastructure** | Database and integrations (SurrealDB, Postgres, Mongo, Stripe, cache, email). |
| **Application** | Orchestration and interfaces (web, api, cli). |
| **Apps** | Independent binaries, each targeting different domains. |

---

## 🧩 Core Principles

- **No panics:** CI forbids `unwrap`, `expect`, `panic!`, `todo!`, `dbg!`.
- **Typed errors:** Distinct `AppError`, `DomainError`, and `InfraError`.
- **Tenant isolation:** Every repository and service enforces `tenant_id`.
- **Observability:** `tracing` spans carry `request_id` + `tenant_id`.
- **Security-first:** Nginx + Cloudflare TLS proxy, no Docker or YAML.
- **Performance:** Compiled HTML/CSS/JS/static assets in binaries.
- **Simplicity:** Minimal dependencies, binary-only distribution.

---

## 🧰 Technology Stack

| Category | Tech |
|-----------|------|
| Language | Rust (edition 2024, panic=abort, LTO=thin) |
| Runtime | Tokio |
| Web | Actix Web + Askama (SSR-first, HTMX optional) |
| Database | SurrealDB (LillPepe), PostgreSQL (Radar), PostgreSQL + MongoDB (BigPepe) |
| Observability | `tracing` |
| Payments | Stripe |
| OS | Parrot OS (dev) / Debian (server) |
| CLI | Fish shell |

---

## ⚙️ Build & Run (Fish Shell)

```fish
# Clone repository
git clone https://github.com/leonardusfisser/maangframe.git
cd maangframe

# Build full workspace (release)
cargo build --release

# Run an example app
cargo run -p lillpepe

Example Tree:
maangframe/
├── apps/
│   ├── lillpepe/
│   ├── radar/
│   ├── streaming/
│   └── bigpepe/
├── crates/
│   ├── core/
│   ├── domain/
│   ├── infra/
│   └── application/
├── ops/
│   ├── archive_maang/
│   ├── backup_tenant/
│   ├── provision_tenant/
│   └── release_switch/
├── assets/
│   ├── images/
│   ├── videos/
│   └── audio/
├── docs/
│   ├── ARCHITECTURE.md
│   ├── DEPLOY.md
│   ├── SECURITY.md
│   └── BACKUP_RESTORE.md
└── Cargo.toml

🔁 Ops Binaries
Binary	Purpose
archive_maang	Archive full workspace to .tar.gz bundles
backup_tenant	Per-tenant SurrealDB backup
provision_tenant	Create new tenant DB + static dirs
release_switch	Rollback / promote release binaries

Automated with systemd timers and netdata for monitoring.

🚦 CI / Build Rules

Deny warnings, forbid unsafe macros

Workspace-wide lints enabled

Release profile:
[profile.release]
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
opt-level = 3

🛡️ Security & Deployment

Cloudflare DNS + TLS termination

Nginx reverse proxy (443 → 127.0.0.1:8080)

Environment loading (priority: CLI > env file > .env.production)

No Docker, no external runtime dependencies

Per-tenant SurrealDB instances (sandboxed)


🌍 Deployment Philosophy

Each app compiles to a standalone binary:

Contains static files, templates, DB schema, config

Isolated tenant data

Runs directly under systemd

Goal: scp binary → enable service → site live.

🧭 Roadmap
Phase	Description	Status
1	Framework foundation (core, infra, domain, application)	✅ Stable
2	LillPepe (CMS SaaS, SurrealDB per tenant)	🚧 In progress
3	Radar (property/auction data aggregator, PostgreSQL)	⏳ Planned
4	Streaming App (live/recorded events)	⏳ Planned
5	BigPepe (real estate / e-commerce SaaS)	⏳ Planned
📜 License

© 2025 Leon Fisser — All rights reserved.
Private closed-source framework.

🧩 Philosophy

“Build once. Ship binary. Run forever.”
— MAANG Principle

Rust. Secure. Fast. Minimal. Tenant-isolated.




# 🦀 MAANGFRAME

**MAANGFRAME** is a secure, multi-tenant Rust framework for building white-label SaaS apps.  
It powers projects like **LillPepe**, **Radar**, and **BigPepe**.

[![Crates.io](https://img.shields.io/crates/v/maangframe.svg)](https://crates.io/crates/maangframe)
[![Docs.rs](https://docs.rs/maangframe/badge.svg)](https://docs.rs/maangframe)
[![Build](https://github.com/leonardusfisser/maangframe/actions/workflows/ci.yml/badge.svg)](https://github.com/leonardusfisser/maangframe/actions/workflows/ci.yml)

---

### ✨ Highlights
- **4-layer architecture**: Web → Application → Domain → Infrastructure  
- **Multi-tenant isolation** (`white_label_id` enforced per repository)  
- **Typed errors**, **observability** (`tracing` spans), **no panics**  
- **Database-agnostic**: SurrealDB, PostgreSQL, MongoDB

---

### 🚀 Docs & Source
# - 📦 [Crates.io](https://crates.io/crates/maangframe)  
# - 📘 [Docs.rs](https://docs.rs/maangframe)  
# - 🏠 [Website](https://maangframe.com)  
# - 💾 [GitHub](https://github.com/leonardusfisser/maangframe)
