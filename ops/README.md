Install the systemd units/timers you committed

# copy unit files into place
sudo install -m0644 ops/etc/systemd/system/backup-tenant@.service /etc/systemd/system/backup-tenant@.service
sudo install -m0644 ops/etc/systemd/system/backup-tenant@.timer   /etc/systemd/system/backup-tenant@.timer
sudo install -m0644 ops/etc/systemd/system/verify-tenants.service /etc/systemd/system/verify-tenants.service
sudo install -m0644 ops/etc/systemd/system/verify-tenants.timer   /etc/systemd/system/verify-tenants.timer

sudo systemctl daemon-reload
sudo systemctl enable --now verify-tenants.timer
# enable backups per tenant when ready:
# sudo systemctl enable --now backup-tenant@alicecakes.com.timer
