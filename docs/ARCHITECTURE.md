# LillPepe Architecture Overview

## Purpose
LillPepe is a multi-tenant SaaS monolith built in Rust (Actix + SurrealDB).
Every tenant has its own folder and SurrealDB namespace.

## Layers
1. **Core** – config, errors, observability, utils
2. **Domain** – business logic (users, tenants, posts, etc.)
3. **Infra** – SurrealDB, email, storage, Stripe, metrics
4. **Application** – Actix web app, CLI, web handlers

## Runtime
- One binary serves all tenants.
- Each tenant has an isolated folder under `/srv/lillpepe/tenants/<domain>/`.
- Each tenant namespace in SurrealDB is `<domain_sanitized>`.
- Caddy routes by Host header.
- systemd template: `lillpepe@<domain>.service`.

## Servers
- LP (web/CMS) → Hetzner EU
- BP (e-commerce) → Hetzner EU
- RD (Radar) → Hetzner EU or region-specific
- Head-Staff EU (control plane) manages all servers.

## Observability
- tracing spans: `{request_id, tenant_id, app_kind, version}`
- metrics endpoint: `/metrics`



NOTES: 
1.PRINCIPLES
production like facebook amazon apple netflix google
safe secure fast minimal libraries low maintenance

2.LAYERED ARCHITECTURE
Production 4-layer Rust architecture
typed errors tenant isolation observability
and CI safety rules.
apps - lillpepe bigpepe

3.CRATE
application - api cli web

core - config errors observability utils
domain - users tenants products services payments photos videos
infra - surrealdb stripe cache storage email metrics

4.DATA FLOW & CONTEXT

5.CORE
Rust, Tokio, Actix-Web, SQLx, Askama, Stripe, HTMX


APP DESCRIPTION
Fully automated
Framework Monolith MAANG that runs Lillpepe
Saas app that sells software licenses to tenants
Binary compiles html css js sql into binary
and shipped automatically together with a database
so that the whole tenant's app is isolated on their domain.
Ship binary to tenant domain with db, external default static files.

TENANT CAN
Change 5 colors, background videos or images,
css file with variables are stored on domain and is self healing.
CRUD products services posts audio
Stores own static files: images, videos, audio
Edit About and Contact page

LIBRARIES
Web Actix, serde, thiserror, tracing

DATABASES
SurrealDB

PAYMENTS
Stripe,

ONLY ON LOGGED IN PAGES
Javascript, Redis, HTMX, Askama

FRONTEND
html, css

I DONT LIKE
GitHub Git Actions Docker Kubernettes YAML Makefile

ABSOLUTELY NOT
unwrap, expect panic, chrono.
Javascript Libraries Frameworks.
CSS Frameworks

NON NEGOTIABLES
production
safe secure
fast
minimal libraries
low maintence code

PAGES
home about products services contact photos videos posts login register admin dashboard

HARDWARE OS
DEV MP80 Parrot OS
SERVER MP100 Debian
CLI = fish

TOOLS
Netdata, rsync, ssh

PREFERENCES
short summarized factual answers, only code when i ask
rust ide db