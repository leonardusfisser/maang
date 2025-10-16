# Operations Manual

## Deployment
1. Build release binary on dev box.
2. rsync to `/opt/lillpepe/releases/<version>/`.
3. Verify checksum, update `/opt/lillpepe/bin/current` symlink.
4. Restart one canary tenant, run health checks.
5. Restart all tenants.

## Backup
Nightly (systemd timer):
- Export SurrealDB NS/DB to `.surql`.
- Tar + zstd `/srv/lillpepe/tenants/<domain>/`.
- Store under `/backups/<domain>/YYYY-MM-DD.tar.zst`.
- Offsite sync to EU backup node.

## Restore
1. Stop tenant service.
2. Restore files + import DB.
3. Start service, check `/health`.

## Health
- `/health` → 200 OK
- CPU < 60 %, RAM free > 25 %
- p95 latency < 250 ms
- 5xx < 0.5 %

## Logs
- App logs: `/var/log/lillpepe/app/`
- Ops logs: `/var/log/lillpepe/ops/`
- Backup logs: `/var/log/lillpepe/backup/`
