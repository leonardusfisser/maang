use deadpool_postgres::Pool;
use domain::{CreateUser, User, UserRole, UserStatus};
use frame::InfraError;
use uuid::Uuid;

pub async fn create_user(
    pool: &Pool,
    create_user: &CreateUser,
    password_hash: &str,
    salt: &str,
) -> Result<User, InfraError> {
    let client = pool.get().await?;

    let id = Uuid::new_v4();  // Keep as Uuid, don't convert to String
    let tenant_uuid = Uuid::parse_str(&create_user.tenant_id)
        .map_err(|e| InfraError::database(format!("Invalid tenant UUID: {e}")))?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let row = client
        .query_one(
            "INSERT INTO users (id, tenant_id, email, password_hash, salt, role, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, tenant_id, email, password_hash, salt, role, status, created_at, updated_at",
            &[&id, &tenant_uuid, &create_user.email, &password_hash, &salt,
                &format!("{:?}", create_user.role).to_lowercase(), &"active", &now, &now],
        )
        .await?;

    Ok(User {
        id: row.get::<_, Uuid>(0).to_string(),
        tenant_id: row.get::<_, Uuid>(1).to_string(),
        email: row.get(2),
        password_hash: row.get(3),
        salt: row.get(4),
        role: parse_role(row.get(5)),
        status: parse_status(row.get(6)),
        created_at: row.get(7),
        updated_at: row.get(8),
    })
}

pub async fn find_by_email(pool: &Pool, email: &str) -> Result<Option<User>, InfraError> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT id, tenant_id, email, password_hash, salt, role, status, created_at, updated_at
             FROM users WHERE email = $1 LIMIT 1",
            &[&email],
        )
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(User {
        id: row.get::<_, Uuid>(0).to_string(),
        tenant_id: row.get::<_, Uuid>(1).to_string(),
        email: row.get(2),
        password_hash: row.get(3),
        salt: row.get(4),
        role: parse_role(row.get(5)),
        status: parse_status(row.get(6)),
        created_at: row.get(7),
        updated_at: row.get(8),
    }))
}

pub async fn find_by_id(pool: &Pool, id: &str) -> Result<Option<User>, InfraError> {
    let client = pool.get().await?;

    let uuid = Uuid::parse_str(id)
        .map_err(|e| InfraError::database(format!("Invalid UUID: {e}")))?;

    let rows = client
        .query(
            "SELECT id, tenant_id, email, password_hash, salt, role, status, created_at, updated_at
             FROM users WHERE id = $1 LIMIT 1",
            &[&uuid],
        )
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(User {
        id: row.get::<_, Uuid>(0).to_string(),
        tenant_id: row.get::<_, Uuid>(1).to_string(),
        email: row.get(2),
        password_hash: row.get(3),
        salt: row.get(4),
        role: parse_role(row.get(5)),
        status: parse_status(row.get(6)),
        created_at: row.get(7),
        updated_at: row.get(8),
    }))
}

fn parse_role(s: &str) -> UserRole {
    match s {
        "admin" => UserRole::Admin,
        "guest" => UserRole::Guest,
        _ => UserRole::User,
    }
}

fn parse_status(s: &str) -> UserStatus {
    match s {
        "inactive" => UserStatus::Inactive,
        "suspended" => UserStatus::Suspended,
        _ => UserStatus::Active,
    }
}