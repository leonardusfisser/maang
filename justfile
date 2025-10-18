# ======================================
# 🦀 MAANG Workspace Commands
# ======================================
set shell := ["fish", "-c"]

# Build all crates in release mode
build:
    cargo build --workspace --release

# Run the main LillPepe app (for local dev)
run-lillpepe:
    cargo run --release --bin lillpepe

# Clean build artifacts safely
clean:
    cargo clean

# Check code for warnings and deny panics
check:
    RUSTFLAGS="-Dwarnings" cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used

# Run tests workspace-wide
test:
    cargo test --workspace

# Archive project (for deployment)
archive:
    ./ops/make_maang_archives.sh

# Backup tenant data
backup:
    ./ops/backup_tenant/target/release/backup_tenant

# Bump version number (from VERSION file)
version:
    @echo (cat VERSION)

# ======================================
# Notes:
# - Use `just <recipe>` to run a command.
# - Fish shell is required (no Bash).
# ======================================

# USAGE:
just build
just run-lillpepe
just archive
just version
