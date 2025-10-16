# BUILD INSTALL
# from apps/lillpepe/ops/release_switch
# cargo build --release
# sudo install -m0755 target/release/release_switch /opt/lillpepe/ops/release_switch


# USAGE:
# normal release
# sudo /opt/lillpepe/ops/release_switch 2025.10.16+001 alicecakes.com --all

# emergency rollback to previous (no version needed)
# sudo /opt/lillpepe/ops/release_switch --rollback
