use anyhow::Result;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use tracing::{Level, info};
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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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
    let store_id = seed_main_store(&mut tx).await?;

    // Seed super admin user
    info!("Seeding super admin user...");
    let super_admin_id = seed_super_admin_user(&mut tx).await?;

    // Assign super admin to main store with super_admin role
    info!("Assigning super admin to main store...");
    seed_super_admin_store_assignment(&mut tx, super_admin_id, store_id, &role_ids).await?;

    // Seed terminal for the main store
    info!("Seeding terminal...");
    let terminal_id = seed_terminal(&mut tx, store_id).await?;

    // Seed CAI range for the terminal
    info!("Seeding CAI range...");
    let cai_range_id = seed_cai_range(&mut tx, terminal_id).await?;

    // Seed tax rates for the main store
    info!("Seeding tax rates...");
    seed_tax_rates(&mut tx, store_id).await?;

    // Seed fiscal sequence for the terminal
    info!("Seeding fiscal sequence...");
    seed_fiscal_sequence(&mut tx, store_id, terminal_id, cai_range_id).await?;

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
            let permission_id = permission_ids
                .get(*permission_code)
                .expect("Permission not found");

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

        info!(
            "  Assigned {} permissions to role: {}",
            permissions.len(),
            role_name
        );
    }

    Ok(())
}

async fn seed_main_store(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Uuid> {
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

    // Get the actual ID (in case it already existed)
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM stores WHERE name = $1")
        .bind(data::MAIN_STORE.0)
        .fetch_one(&mut **tx)
        .await?;

    Ok(row.0)
}

async fn seed_super_admin_user(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<Uuid> {
    let id = Uuid::now_v7();

    // Hash the password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(data::SUPER_ADMIN_USER.4.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO users (id, email, username, first_name, last_name, password_hash, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        ON CONFLICT (email) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(data::SUPER_ADMIN_USER.0) // email
    .bind(data::SUPER_ADMIN_USER.1) // username
    .bind(data::SUPER_ADMIN_USER.2) // first_name
    .bind(data::SUPER_ADMIN_USER.3) // last_name
    .bind(&password_hash)
    .execute(&mut **tx)
    .await?;

    info!(
        "  User: {} ({})",
        data::SUPER_ADMIN_USER.1,
        data::SUPER_ADMIN_USER.0
    );

    // Get the actual ID (in case it already existed)
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(data::SUPER_ADMIN_USER.0)
        .fetch_one(&mut **tx)
        .await?;

    Ok(row.0)
}

async fn seed_super_admin_store_assignment(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    store_id: Uuid,
    role_ids: &std::collections::HashMap<String, Uuid>,
) -> Result<()> {
    // First, create user_stores relationship (composite PK: user_id, store_id)
    sqlx::query(
        r#"
        INSERT INTO user_stores (user_id, store_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, store_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(store_id)
    .execute(&mut **tx)
    .await?;

    // Assign super_admin role (composite PK: user_id, store_id, role_id)
    let super_admin_role_id = role_ids
        .get("super_admin")
        .expect("super_admin role not found");

    sqlx::query(
        r#"
        INSERT INTO user_store_roles (user_id, store_id, role_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, store_id, role_id) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(store_id)
    .bind(super_admin_role_id)
    .execute(&mut **tx)
    .await?;

    info!("  Assigned super_admin role to user in store");

    Ok(())
}

async fn seed_terminal(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<Uuid> {
    let id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO terminals (id, store_id, code, name, is_active)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(store_id)
    .bind("CAJA-001")
    .bind("Caja Principal")
    .bind(true)
    .execute(&mut **tx)
    .await?;

    // Get actual ID
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM terminals WHERE store_id = $1 AND code = $2")
        .bind(store_id)
        .bind("CAJA-001")
        .fetch_one(&mut **tx)
        .await?;

    info!("  Terminal: CAJA-001 (Caja Principal)");
    Ok(row.0)
}

async fn seed_cai_range(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    terminal_id: Uuid,
) -> Result<Uuid> {
    let id = Uuid::now_v7();
    let expiry = NaiveDate::from_ymd_opt(2027, 12, 31).unwrap();

    sqlx::query(
        r#"
        INSERT INTO cai_ranges (id, terminal_id, cai_number, range_start, range_end, current_number, expiration_date, is_exhausted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(terminal_id)
    .bind("A1B2C3-D4E5F6-G7H8I9-J0K1L2-M3N4O5-P6")
    .bind(1_i64)
    .bind(50000_i64)
    .bind(1_i64)
    .bind(expiry)
    .bind(false)
    .execute(&mut **tx)
    .await?;

    // Get actual ID
    let row: (Uuid,) = sqlx::query_as(
        "SELECT id FROM cai_ranges WHERE terminal_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(terminal_id)
    .fetch_one(&mut **tx)
    .await?;

    info!("  CAI: A1B2C3-...P6 (rango 1-50000, expira 2027-12-31)");
    Ok(row.0)
}

async fn seed_tax_rates(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    let tax_rates = [
        ("ISV 15%", "isv_15", "0.1500", true, "all"),
        ("ISV 18%", "isv_18", "0.1800", false, "categories"),
        ("Exento", "exempt", "0.0000", false, "categories"),
    ];

    for (name, tax_type, rate, is_default, applies_to) in &tax_rates {
        let id = Uuid::now_v7();
        let rate_decimal: rust_decimal::Decimal = rate.parse().unwrap();

        sqlx::query(
            r#"
            INSERT INTO tax_rates (id, store_id, name, tax_type, rate, is_default, is_active, applies_to, category_ids)
            VALUES ($1, $2, $3, $4, $5, $6, true, $7, '{}')
            ON CONFLICT (store_id, name) DO NOTHING
            "#,
        )
        .bind(id)
        .bind(store_id)
        .bind(name)
        .bind(tax_type)
        .bind(rate_decimal)
        .bind(is_default)
        .bind(applies_to)
        .execute(&mut **tx)
        .await?;

        info!("  Tax Rate: {} ({} = {})", name, tax_type, rate);
    }

    Ok(())
}

async fn seed_fiscal_sequence(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
    terminal_id: Uuid,
    cai_range_id: Uuid,
) -> Result<()> {
    let id = Uuid::now_v7();

    sqlx::query(
        r#"
        INSERT INTO fiscal_sequences (id, store_id, terminal_id, cai_range_id, prefix, current_number, range_start, range_end, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (terminal_id, cai_range_id) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(store_id)
    .bind(terminal_id)
    .bind(cai_range_id)
    .bind("000-001-01-")
    .bind(0_i64)
    .bind(1_i64)
    .bind(50000_i64)
    .bind(true)
    .execute(&mut **tx)
    .await?;

    info!("  Fiscal Sequence: 000-001-01- (rango 1-50000)");
    Ok(())
}
