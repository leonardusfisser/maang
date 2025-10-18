maang/
├── apps/ # Binary crates (lillpepe, etc.)
├── crates/ # Core, Domain, Infra, Application layers
├── ops/ # Operational binaries (backup_tenant, archive_maang, etc.)
├── docs/ # Documentation (this folder)
├── assets/ # Static images, audio, pdf, etc.
├── deploy/ # Systemd, nginx
├── dist/ # Release builds for tenants
├── var/ # Runtime data (backups, logs, tmp)
├── wasm/ # WebAssembly crates


## Folder Rules
- `src/` inside each crate contains code only (no logic in `lib.rs`)
- No nested modules beyond `src/`
- Public APIs exposed via crate-level `lib.rs` re-exports
- All paths relative and portable (`PathBuf` only, no `String` paths)
