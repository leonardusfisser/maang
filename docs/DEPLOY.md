# 🚀 Deployment Guide

## Overview
MAANG apps deploy as **single Rust binaries** behind Nginx + Cloudflare Tunnel on Debian 13 servers.

## Prerequisites
- Debian 13 (server)
- Rust 1.80+
- SurrealDB (per-tenant)
- NetData (metrics)
- Systemd + SSH + rsync

## Steps
```fish
cargo build --release --bin lillpepe
rsync -av target/release/lillpepe debian:/srv/maang/lillpepe/
rsync -av deploy/nginx/lillpepe.conf debian:/etc/nginx/sites-enabled/
sudo systemctl restart nginx
sudo systemctl restart lillpepe@tenant1
