use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

mod data;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Starting seed process...");

    // Run seeds in transaction
    let mut tx = pool.begin().await?;

    // Seed permissions
    info!("Seeding permissions...");
    let permission_ids = seed_permissions(&mut tx).await?;

    // Seed roles
    info!("Seeding roles...");
    let role_ids = seed_roles(&mut tx).await?;

    // Seed role-permission relationships
    info!("Seeding role permissions...");
    seed_role_permissions(&mut tx, &role_ids, &permission_ids).await?;

    // Seed main store
    info!("Seeding main store...");
    seed_main_store(&mut tx).await?;

    tx.commit().await?;

    info!("Seed completed successfully!");
    Ok(())
}

async fn seed_permissions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<std::collections::HashMap<String, Uuid>> {
    let mut permission_ids = std::collections::HashMap::new();

    for (code, description) in data::PERMISSIONS {
        let id = Uuid::now_v7();
        
        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, description)
            VALUES ($1, $2, $3)
            ON CONFLICT (code) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(code)
        .bind(description)
        .execute(&mut **tx)
        .await?;

        // Get the actual ID (in case it already existed)
        let row: (Uuid,) = sqlx::query_as("SELECT id FROM permissions WHERE code = $1")
            .bind(code)
            .fetch_one(&mut **tx)
            .await?;

        permission_ids.insert(code.to_string(), row.0);
        info!("  Permission: {}", code);
    }

    Ok(permission_ids)
}

async fn seed_roles(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<std::collections::HashMap<String, Uuid>> {
    let mut role_ids = std::collections::HashMap::new();

    for (name, description, is_system_protected) in data::ROLES {
        let id = Uuid::now_v7();

        sqlx::query(
            r#"
            INSERT INTO roles (id, name, description, is_system_protected)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(is_system_protected)
        .execute(&mut **tx)
        .await?;

        // Get the actual ID (in case it already existed)
        let row: (Uuid,) = sqlx::query_as("SELECT id FROM roles WHERE name = $1")
            .bind(name)
            .fetch_one(&mut **tx)
            .await?;

        role_ids.insert(name.to_string(), row.0);
        info!("  Role: {}", name);
    }

    Ok(role_ids)
}


async fn seed_role_permissions(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    role_ids: &std::collections::HashMap<String, Uuid>,
    permission_ids: &std::collections::HashMap<String, Uuid>,
) -> Result<()> {
    for (role_name, permissions) in data::ROLE_PERMISSIONS {
        let role_id = role_ids.get(*role_name).expect("Role not found");

        for permission_code in *permissions {
            let permission_id = permission_ids.get(*permission_code).expect("Permission not found");

            sqlx::query(
                r#"
                INSERT INTO role_permissions (role_id, permission_id)
                VALUES ($1, $2)
                ON CONFLICT (role_id, permission_id) DO NOTHING
                "#,
            )
            .bind(role_id)
            .bind(permission_id)
            .execute(&mut **tx)
            .await?;
        }

        info!("  Assigned {} permissions to role: {}", permissions.len(), role_name);
    }

    Ok(())
}

async fn seed_main_store(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<()> {
    let id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO stores (id, name, address, is_ecommerce, is_active)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(data::MAIN_STORE.0)
    .bind(data::MAIN_STORE.1)
    .bind(data::MAIN_STORE.2)
    .bind(data::MAIN_STORE.3)
    .execute(&mut **tx)
    .await?;

    info!("  Store: {}", data::MAIN_STORE.0);

    Ok(())
}
