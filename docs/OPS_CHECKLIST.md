# Operations Checklist

## Pre-Release
- [ ] All tests pass
- [ ] No warnings (`RUSTFLAGS="-D warnings"`)
- [ ] Cargo.lock updated
- [ ] Version bumped
- [ ] Docs archived to `/var/archives/`

## Post-Release
- [ ] Run `ops/smoke_test.sh`
- [ ] Verify `/health` endpoint
- [ ] Inspect `var/logs/dev` and `prod`
- [ ] Confirm backups executed
- [ ] Update CHANGELOG.md
