use deadpool_postgres::Pool;
use domain::{Service, ServiceStatus};
use frame::InfraError;
use uuid::Uuid;

pub async fn create_service(
    pool: &Pool,
    tenant_id: &str,
    name: &str,
    description: &str,
) -> Result<Service, InfraError> {
    let client = pool.get().await?;

    let id = Uuid::new_v4();
    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|e| InfraError::database(format!("Invalid tenant UUID: {e}")))?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let row = client
        .query_one(
            "INSERT INTO services (id, tenant_id, name, description, status, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, '{}', $6, $7)
             RETURNING id, tenant_id, name, description, status, metadata, created_at, updated_at",
            &[&id, &tenant_uuid, &name, &description, &"active", &now, &now],
        )
        .await?;

    Ok(Service {
        id: row.get::<_, Uuid>(0).to_string(),
        tenant_id: row.get::<_, Uuid>(1).to_string(),
        name: row.get(2),
        description: row.get(3),
        status: ServiceStatus::Active,
        metadata: row.get(5),
        created_at: row.get(6),
        updated_at: row.get(7),
    })
}

pub async fn list_services(
    pool: &Pool,
    tenant_id: &str,
) -> Result<Vec<Service>, InfraError> {
    let client = pool.get().await?;

    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|e| InfraError::database(format!("Invalid tenant UUID: {e}")))?;

    let rows = client
        .query(
            "SELECT id, tenant_id, name, description, status, metadata, created_at, updated_at
             FROM services WHERE tenant_id = $1 AND status = 'active' ORDER BY created_at DESC",
            &[&tenant_uuid],
        )
        .await?;

    let services = rows
        .iter()
        .map(|row| Service {
            id: row.get::<_, Uuid>(0).to_string(),
            tenant_id: row.get::<_, Uuid>(1).to_string(),
            name: row.get(2),
            description: row.get(3),
            status: ServiceStatus::Active,
            metadata: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7),
        })
        .collect();

    Ok(services)
}