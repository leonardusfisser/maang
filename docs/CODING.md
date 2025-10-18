Retention: Keep last 5 backups; purge older.

## Rust Edition
- 2024
- Resolver = 3

## Style Rules
- No panics, unwraps, expects
- All functions return `Result<T, E>`
- Modules are flat, no nested `mod.rs`
- Each crate exports only what is public
- Use `tracing` for logging; no `println!`
- Use `thiserror` for typed errors

## Formatting
```fish
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
