Observability

Every request span includes:

request_id
tenant_id
user_id (if authenticated)

## Naming
- Crates: snake_case (e.g., `core_config`)
- Modules: singular nouns
- Structs: PascalCase
- Functions: snake_case
- Errors: `XxxError`

## Paths
- Always use `PathBuf`, never hardcoded `/` strings.
- Relative paths only, resolved at runtime.

## Logging
Use `info!`, `warn!`, `error!`, never `println!`.

## Commits
Prefix with context:

core: add config validator
domain: fix tenant repo error mapping
infra: refactor surrealdb connection pool