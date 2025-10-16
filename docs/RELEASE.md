# Release Procedure

1. Build on dev machine: `cargo build --release`.
2. Create version folder, checksum.
3. rsync to each server under `/opt/lillpepe/releases/<version>/`.
4. Verify sha256sum.
5. Switch symlink `/opt/lillpepe/bin/current`.
6. Restart canary tenant.
7. Verify health; then restart all tenants.
8. Record version in `/opt/lillpepe/VERSION`.
9. Keep last 3 versions; remove older.

Rollback:
