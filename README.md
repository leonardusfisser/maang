# Full production validation
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all -- --check
cargo deny check
cargo audit
```

**Add to root `.gitattributes`:**
```
*.rs text eol=lf
*.toml text eol=lf
*.md text eol=lf
*.sh text eol=lf