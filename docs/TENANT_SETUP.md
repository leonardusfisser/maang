# 🧩 Tenant Setup Guide

## Overview
Each tenant gets:
- Its own SurrealDB file
- Configured domain (via Cloudflare)
- Compiled binary with tenant env vars

## Create a Tenant
```fish
./ops/provision_tenant tenant1

Migration:
./ops/migrate_tenant tenant1

Delete:
./ops/delete_tenant tenant1

Verification:
./ops/verify_tenants

Tenant Paths
/var/tenants/<tenant_id>/
/var/db/<tenant_id>.surreal

