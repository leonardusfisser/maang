# Security Policy

## Data Handling
- Passwords: Argon2id one-way hash.
- PII (email, phone, address): AES-GCM 256-bit encryption.
- Encryption key: `/etc/lillpepe/keys/platform.key` (0600 perms).

## Keys & Secrets
- Never in tenant folders or backups.
- Rotate quarterly via `rotate-keys` ops binary.

## Network
- All servers TLS via Caddy.
- SurrealDB binds 127.0.0.1 only.
- SSH keys only, no password login.

## Backups
- Encrypted at rest.
- Offsite copies EU-only.
- Restore tested monthly.

## Incident Response
1. Detect → contain → analyse.
2. Notify tenants within 72h if breach confirmed.
3. Rotate affected keys.
4. Document and store report.
