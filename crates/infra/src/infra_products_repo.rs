use deadpool_postgres::Pool;
use domain::{CreateProduct, Product, ProductStatus};
use frame::InfraError;
use uuid::Uuid;

pub async fn create_product(
    pool: &Pool,
    create: &CreateProduct,
) -> Result<Product, InfraError> {
    let client = pool.get().await?;

    let id = Uuid::new_v4();
    let tenant_uuid = Uuid::parse_str(&create.tenant_id)
        .map_err(|e| InfraError::database(format!("Invalid tenant UUID: {e}")))?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();

    let row = client
        .query_one(
            "INSERT INTO products (id, tenant_id, name, description, price, currency, status, image_url, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, '{}', $8, $9)
             RETURNING id, tenant_id, name, description, price, currency, status, image_url, metadata, created_at, updated_at",
            &[&id, &tenant_uuid, &create.name, &create.description, &create.price, &create.currency, &"active", &now, &now],
        )
        .await?;

    Ok(Product {
        id: row.get::<_, Uuid>(0).to_string(),
        tenant_id: row.get::<_, Uuid>(1).to_string(),
        name: row.get(2),
        description: row.get(3),
        price: row.get(4),
        currency: row.get(5),
        status: ProductStatus::Active,
        image_url: row.get(7),
        metadata: row.get(8),
        created_at: row.get(9),
        updated_at: row.get(10),
    })
}

pub async fn list_products(
    pool: &Pool,
    tenant_id: &str,
) -> Result<Vec<Product>, InfraError> {
    let client = pool.get().await?;

    let tenant_uuid = Uuid::parse_str(tenant_id)
        .map_err(|e| InfraError::database(format!("Invalid tenant UUID: {e}")))?;

    let rows = client
        .query(
            "SELECT id, tenant_id, name, description, price, currency, status, image_url, metadata, created_at, updated_at
             FROM products WHERE tenant_id = $1 AND status = 'active' ORDER BY created_at DESC",
            &[&tenant_uuid],
        )
        .await?;

    let products = rows
        .iter()
        .map(|row| Product {
            id: row.get::<_, Uuid>(0).to_string(),
            tenant_id: row.get::<_, Uuid>(1).to_string(),
            name: row.get(2),
            description: row.get(3),
            price: row.get(4),
            currency: row.get(5),
            status: ProductStatus::Active,
            image_url: row.get(7),
            metadata: row.get(8),
            created_at: row.get(9),
            updated_at: row.get(10),
        })
        .collect();

    Ok(products)
}