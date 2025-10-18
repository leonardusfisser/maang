# 🧑‍💻 MAANG FRAMEWORK — Developer Guide

This document is for internal development of the **MAANG Monorepo**.  
It explains environment setup, build process, linting, CI rules, ops tooling, and conventions.

---

## 🧱 Development Environment

**OS:**
- Dev: Parrot OS (fish shell)
- Prod: Debian (systemd services)

**Requirements:**
```fish
sudo apt update; and sudo apt install -y build-essential libssl-dev pkg-config
curl https://sh.rustup.rs -sSf | sh
rustup default stable
cargo install cargo-audit cargo-deny cargo-outdated


Recommended Tools:

fish shell for scripting

netdata for node metrics

rsync + ssh for deployment

tracing-subscriber for local logs

🧩 Workspace Layout


maangframe/
├── apps/              # Independent apps (LillPepe, Radar, Streaming, BigPepe)
├── crates/            # Shared framework
│   ├── core/          # Config, errors, observability, utils
│   ├── domain/        # Business logic
│   ├── infra/         # Database + external services
│   └── application/   # Orchestration layer
├── ops/               # Maintenance binaries (backup, archive, release)
├── assets/            # Static assets (html, css, js, media)
├── docs/              # Architecture, deploy, security, etc.
├── README.md
├── README.dev.md
└── Cargo.toml


⚙️ Build & Run

Build everything:

cargo build --release


Run a specific app (e.g. LillPepe):

cargo run -p lillpepe


Run ops tools:

cargo run -p backup_tenant
cargo run -p archive_maang

🧪 Testing & Linting

Run all tests:

cargo test --workspace


Run Clippy lints (CI strict):

cargo clippy --workspace --all-targets -- -D warnings


Check for outdated crates:

cargo outdated


Security audit:

cargo audit

🚨 CI Safety Rules

The workspace denies unsafe operations:

[lints]
workspace = true

[profile.release]
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
opt-level = 3

Forbidden in all crates:

unwrap

expect

panic!

todo!

dbg!

Mandatory:

Typed errors (AppError, DomainError, InfraError)

tracing spans with request_id, tenant_id

🧱 Environment Configuration

Priority loading order:

CLI arguments

.env file (dev only)

.env.production (fallback)

Required variables:

APP_ENV=development|production
HOST=127.0.0.1
PORT=8080


Each app defines additional environment keys in its config crate.

🧩 Tenant Isolation

Every app and repository enforces tenant_id:

Required in all CRUD operations

Injected in tracing spans

Ensures data separation for multi-tenant deployments

SurrealDB (LillPepe) creates one database per tenant in /var/lib/maang/tenants/<tenant_id>.

📦 Ops Workflow

Each binary in /ops automates server maintenance.

Binary	Purpose
archive_maang	Compress full repo → .tar.gz
backup_tenant	Backup SurrealDB tenant data
provision_tenant	Initialize new tenant DB + static dirs
release_switch	Swap production binary safely

Run manually or via systemd timers.

🌍 Deployment Checklist

Compile release binary:

cargo build --release -p lillpepe


Copy binary to server:

rsync -av target/release/lillpepe debian:/opt/maang/apps/


Restart systemd service:

ssh debian "sudo systemctl restart lillpepe.service"


Verify:

curl -fsS https://tenant.example.com/health

🔍 Observability

Uses tracing + tracing-subscriber.

Each request logs:

[request_id=abc123 tenant_id=tenant42 level=INFO]


To enable verbose tracing:

RUST_LOG=trace cargo run -p lillpepe

🧭 Developer Roadmap
Phase	Description	Status
1	Framework crates complete	✅
2	Docs normalized (10+ files)	🚧
3	Auth + tenant CRUD implementation	⏳
4	Payments + media uploads	⏳
5	Multi-app CI pipeline	⏳
📜 Conventions

All code must compile without warnings

Small, self-contained modules per crate

No logic in lib.rs or mod.rs

SSR-first (Askama templates)

Minimal dependencies, no JavaScript frameworks

Each crate uses typed error enums and Result<T, E>

🧩 Philosophy

“Secure, typed, tenant-isolated Rust apps that run forever.”

Fast. Minimal. Maintainable. Binary-first.


---

✅ **Answer:**  
Place `README.dev.md` in the **root folder** beside `README.md`.  
That’s the conventional and discoverable spot for developer documentation in Rust monorepos.