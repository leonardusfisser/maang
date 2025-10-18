# [workspace.metadata.*] # your own config buckets
# Arbitrary key–value space read by tools/xtask/build scripts. Great for centralizing versions, paths, release channels, etc., without affecting Cargo itself.
# Examples:
# workspace.metadata.release — notes for your release script (channels, changelog paths).
# workspace.metadata.paths — canonical dirs (backups, artifacts).
# workspace.metadata.apps — app list + human names (for generators or dashboards).
# This keeps “project constants” in one place and avoids hardcoding in scripts.


# Concrete dependency guidance (fits your policy)

# Add:
# surrealdb = "<stable>" (used only by LillPepe or shared infra if you abstract it)
# Keep Postgres + Mongo support in the framework (for other apps):

# Postgres (lean):
# tokio-postgres = "<stable>"
# bb8 = "<stable>" and bb8-postgres = "<stable>" or deadpool-postgres = "<stable>"

# Postgres (typed):
# sqlx = { version = "0.7", default-features = false, features = ["runtime-tokio-rustls","postgres","time"] }

# Mongo:
# mongodb = { version = "<stable>", default-features = false, features = ["tokio-runtime"] }



