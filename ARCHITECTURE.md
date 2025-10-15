1.PRINCIPLES
production like facebook amazon apple netflix google
safe secure fast minimal libraries low maintenance 

2.LAYERED ARCHITECTURE
Production 4-layer Rust architecture, typed errors, tenant isolation, observability, and CI safety rules.

3.CRATE
application - application, web, api, cli
apps - lillpepe 
core - config, errors, observability, utils
domain - users, tenants, products, services, payments, photos, videos
infrastructure - postgress, mongodb, surrealdb, stripe, cache, storage, email, metrics 

4.DATA FLOW & CONTEXT

5.CORE
Rust, Tokio, Actix-Web, SQLx, Askama, Stripe, HTMX


CRATES

APP DESCRIPTION
Fully automated
Monolith Lillpepe Saas app that sells webapps.
Compile html css js sql into binary
Fully functional binary with db and payment integration
Ship binary to tenant domain with db, external default static files

TENANT CAN
Change 5 colors, background videos or images,
CRUD products services posts audio
Stores own static files: images, videos, audio
Edit About and Contact page

LIBRARIES
Web Actix, serde, thiserror

DATABASES
Postgress (core), MongoDB (docs), SurrealDB (mirror)

PAYMENTS
Stripe,

IF I MUST
Javascript, Redis, HTMX, Askama

FRONTEND
html, css

NOT KEEN ON
Git Git-Hub Git Actions Docker Kubernettes YAML

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