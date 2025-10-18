Cloudflare Tunnel

Each tenant runs under its own subdomain with TLS managed by Cloudflare.

## Purpose
How to operate, troubleshoot, and recover the MAANG framework.

## Common Tasks
| Task | Command |
|------|----------|
| Restart tenant service | `sudo systemctl restart lillpepe@tenant1` |
| Check logs | `journalctl -u lillpepe@tenant1 -n 100` |
| Backup tenant | `./ops/backup_tenant --tenant tenant1` |
| Verify tenants | `./ops/verify_tenants` |
| Switch release | `./ops/release_switch` |

## Troubleshooting
- 500 errors → check `var/logs/prod/`
- Tenant db locked → restart SurrealDB instance
- Missing env var → check `/etc/systemd/system/lillpepe@.service` EnvironmentFile

## Incident Response
1. Identify tenant & timestamp  
2. Restore backup from `/var/backups/tenant_name/`  
3. Verify with `verify_tenants`  
4. Update release symlink  
5. Document in `/var/system/incidents.log`
