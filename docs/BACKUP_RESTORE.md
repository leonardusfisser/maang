# 💾 Backup & Restore

## Backup
Uses `rsync` via `ops/backup_tenant`.

```fish
./ops/backup_tenant

Backups stored in:
rsync -av maang_2025-10-17_19-00/ /home/leon/maang/

Verification:
./ops/verify_tenants

