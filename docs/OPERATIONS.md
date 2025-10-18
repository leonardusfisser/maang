## Daily Tasks
- Check systemd timers (`verify-tenants.timer`, `backup-tenant@.timer`)
- Monitor NetData dashboards
- Review `var/logs/prod/`
- Rotate backups weekly

## Weekly Tasks
- Run `./ops/archive_maang`
- Verify tenant consistency
- Apply system updates (`sudo apt update; sudo apt upgrade -y`)

## Monthly
- Rotate Stripe API keys
- Update Cloudflare certificates
