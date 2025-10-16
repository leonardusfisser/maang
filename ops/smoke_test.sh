# build all ops binaries (on server or cross-compile & copy)
for p in backup_tenant verify_tenants provision_tenant release_switch archive_maang; do
  (cd ops/$p && cargo build --release)
  sudo install -m0755 ops/$p/target/release/$p /opt/lillpepe/ops/$p
done

# create archives dir
sudo mkdir -p /opt/lillpepe/archives

# make a platform archive (uses /opt/lillpepe/VERSION if set)
sudo /opt/lillpepe/scripts/make_maang_archives.sh 2025.10.16+001

# verify tenants (should print ✅ or ⚠️)
sudo /opt/lillpepe/ops/verify_tenants
