1.CODE STYLE
no logic in lib.rs and mod.rs

2.ASYNCHRONOUS
tokio

3.DEPENDENCIES
serde, time
minimal dependencies for low maintenance

NO CHRONO
NO ANYHOW

4.OBSERVABILTIY
logging,tracing
user_id, tenant_id

5.DATABASE
postgres (main)
mongodb (docs)
surrealdb (mirror)