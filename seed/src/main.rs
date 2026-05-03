use anyhow::Result;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use sqlx::postgres::PgPoolOptions;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use uuid::{NoContext, Timestamp, Uuid};

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
    // Retry connect a few times — when launched by docker-compose, postgres
    // may still be coming up even after the healthcheck passed.
    let pool = {
        let mut attempts = 0;
        loop {
            match PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(std::time::Duration::from_secs(3))
                .connect(&database_url)
                .await
            {
                Ok(p) => break p,
                Err(e) if attempts < 30 => {
                    attempts += 1;
                    info!(
                        "  database not ready ({}); retrying in 2s ({}/30)",
                        e, attempts
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    };

    // Run migrations embedded at compile time. Idempotent: sqlx tracks applied
    // versions in `_sqlx_migrations`. Putting this here means the seed binary
    // is the single boot-time entry point — no separate sqlx-cli step.
    info!("Running migrations...");
    sqlx::migrate!("../migrations").run(&pool).await?;

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

    // Seed default Manual payment gateway for the main store. Honduras-friendly
    // out of the box: charges go in `pending` until a manager confirms.
    info!("Seeding payment gateway (Manual)...");
    seed_payment_gateway(&mut tx, store_id).await?;

    // Seed shipping defaults: StorePickup + OwnDelivery + Tegucigalpa zone + rates.
    info!("Seeding shipping defaults...");
    seed_shipping_defaults(&mut tx, store_id).await?;

    // Seed default LocalServer image storage provider for the main store.
    info!("Seeding catalog defaults...");
    seed_catalog_defaults(&mut tx, store_id).await?;

    // Seed accounting defaults: HN-aligned chart of accounts + an initial
    // open period for the current month.
    info!("Seeding accounting defaults...");
    seed_accounting_defaults(&mut tx).await?;

    // Seed demand-planning demo: a vendor, ~9 grocery products with stock,
    // reorder policies, and 120 days of synthetic completed sales so the
    // recompute job has signal on first run.
    info!("Seeding demand planning demo data...");
    seed_demand_planning_demo(&mut tx, store_id, super_admin_id).await?;

    // Seed a default bank account so cash_management endpoints have somewhere
    // to record deposits/transactions on first boot.
    info!("Seeding cash management defaults...");
    seed_cash_management_defaults(&mut tx, store_id).await?;

    // Seed a loyalty program (Bronze/Silver/Gold + 2 rewards) for the main
    // store so the loyalty endpoints have a complete graph on first boot.
    info!("Seeding loyalty defaults...");
    seed_loyalty_defaults(&mut tx, store_id).await?;

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

async fn seed_payment_gateway(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    let id = Uuid::now_v7();

    // The Manual adapter ignores credentials — these placeholder values just
    // satisfy the NOT NULL constraints. Real adapters (Stripe, BAC, …) will
    // need encrypted secrets here.
    let supported_methods: Vec<String> = vec![
        "cash".to_string(),
        "bank_transfer".to_string(),
        "cash_on_delivery".to_string(),
        "agency_deposit".to_string(),
    ];
    let supported_currencies: Vec<String> = vec!["HNL".to_string(), "USD".to_string()];

    sqlx::query(
        r#"
        INSERT INTO payment_gateways (
            id, store_id, name, gateway_type, is_active, is_default,
            api_key_encrypted, secret_key_encrypted, merchant_id, is_sandbox,
            supported_methods, supported_currencies, webhook_secret
        )
        VALUES ($1, $2, $3, 'manual', true, true, $4, $5, NULL, false, $6, $7, NULL)
        ON CONFLICT (store_id, name) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(store_id)
    .bind("Caja Manual (HN)")
    .bind("manual-not-applicable")
    .bind("manual-not-applicable")
    .bind(&supported_methods)
    .bind(&supported_currencies)
    .execute(&mut **tx)
    .await?;

    info!("  Payment Gateway: Caja Manual (HN) — manual, default");
    Ok(())
}

async fn seed_shipping_defaults(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    // Two methods covering the most common HN scenarios out of the box.
    sqlx::query(
        r#"INSERT INTO shipping_methods
           (id, store_id, name, code, method_type, description, sort_order)
           VALUES ($1, $2, $3, $4, 'store_pickup', $5, 0)
           ON CONFLICT (store_id, code) DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(store_id)
    .bind("Retiro en tienda")
    .bind("pickup")
    .bind("El cliente recoge el pedido directamente en la tienda")
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"INSERT INTO shipping_methods
           (id, store_id, name, code, method_type, description,
            estimated_days_min, estimated_days_max, sort_order)
           VALUES ($1, $2, $3, $4, 'own_delivery', $5, 0, 1, 1)
           ON CONFLICT (store_id, code) DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(store_id)
    .bind("Envío con motorista propio")
    .bind("own_delivery")
    .bind("Repartido por un conductor de la tienda dentro de Tegucigalpa")
    .execute(&mut **tx)
    .await?;
    info!("  Shipping methods: Retiro en tienda + Envío con motorista propio");

    sqlx::query(
        r#"INSERT INTO shipping_zones
           (id, store_id, name, countries, states, zip_codes)
           VALUES ($1, $2, 'Tegucigalpa', $3, $4, '{}')
           ON CONFLICT DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(store_id)
    .bind(vec!["HN".to_string()])
    .bind(vec!["FM".to_string()])
    .execute(&mut **tx)
    .await?;
    info!("  Shipping zone: Tegucigalpa (HN/FM)");

    // Resolve real ids for the rates.
    let pickup_id: (Uuid,) = sqlx::query_as(
        "SELECT id FROM shipping_methods WHERE store_id = $1 AND code = 'pickup' LIMIT 1",
    )
    .bind(store_id)
    .fetch_one(&mut **tx)
    .await?;
    let own_id: (Uuid,) = sqlx::query_as(
        "SELECT id FROM shipping_methods WHERE store_id = $1 AND code = 'own_delivery' LIMIT 1",
    )
    .bind(store_id)
    .fetch_one(&mut **tx)
    .await?;
    let zone_id: (Uuid,) = sqlx::query_as(
        "SELECT id FROM shipping_zones WHERE store_id = $1 AND name = 'Tegucigalpa' LIMIT 1",
    )
    .bind(store_id)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        r#"INSERT INTO shipping_rates
           (id, shipping_method_id, shipping_zone_id, rate_type,
            base_rate, per_kg_rate, free_shipping_threshold, currency)
           VALUES ($1, $2, $3, 'flat', 0, 0, NULL, 'HNL')
           ON CONFLICT DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(pickup_id.0)
    .bind(zone_id.0)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        r#"INSERT INTO shipping_rates
           (id, shipping_method_id, shipping_zone_id, rate_type,
            base_rate, per_kg_rate, free_shipping_threshold, currency)
           VALUES ($1, $2, $3, 'order_based', 50.00, 0, 1000.00, 'HNL')
           ON CONFLICT DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(own_id.0)
    .bind(zone_id.0)
    .execute(&mut **tx)
    .await?;
    info!("  Shipping rates: Pickup gratis, OwnDelivery L 50 (gratis sobre L 1000)");

    Ok(())
}

async fn seed_catalog_defaults(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    // Default LocalServer image storage provider. The adapter ignores the
    // credentials — they're placeholders to satisfy the NOT NULL constraints.
    sqlx::query(
        r#"INSERT INTO image_storage_providers
          (id, store_id, name, provider_type, is_active, is_default,
           api_key_encrypted, secret_key_encrypted, config_json)
          VALUES ($1, $2, 'Almacenamiento local', 'local_server', true, true,
                  'local-not-applicable', 'local-not-applicable', NULL)
          ON CONFLICT (store_id, name) DO NOTHING"#,
    )
    .bind(Uuid::now_v7())
    .bind(store_id)
    .execute(&mut **tx)
    .await?;

    info!("  Image storage provider: Almacenamiento local (LocalServer, default)");
    Ok(())
}

/// Seeds the default chart of accounts (HN-aligned SME plan) and opens an
/// initial accounting period for the current month. Idempotent — running
/// the seed twice does not create duplicates.
async fn seed_accounting_defaults(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<()> {
    // Chart of accounts.
    for (code, name, account_type) in data::CHART_OF_ACCOUNTS {
        sqlx::query(
            r#"
            INSERT INTO chart_of_accounts (id, code, name, account_type, parent_id, is_active)
            VALUES ($1, $2, $3, $4, NULL, TRUE)
            ON CONFLICT (code) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v7(Timestamp::now(NoContext)))
        .bind(*code)
        .bind(*name)
        .bind(*account_type)
        .execute(&mut **tx)
        .await?;
    }
    info!(
        "  Chart of accounts seeded: {} accounts",
        data::CHART_OF_ACCOUNTS.len()
    );

    // Initial open period covering the current calendar month.
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };

    let starts_at = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
    let ends_at = Utc
        .with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .unwrap();

    let month_name = match month {
        1 => "Enero",
        2 => "Febrero",
        3 => "Marzo",
        4 => "Abril",
        5 => "Mayo",
        6 => "Junio",
        7 => "Julio",
        8 => "Agosto",
        9 => "Septiembre",
        10 => "Octubre",
        11 => "Noviembre",
        _ => "Diciembre",
    };
    let period_name = format!("{} {}", month_name, year);

    // Skip if a period already covers `now`.
    let exists: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id FROM accounting_periods
        WHERE starts_at <= $1 AND ends_at > $1
        LIMIT 1
        "#,
    )
    .bind(now)
    .fetch_optional(&mut **tx)
    .await?;

    if exists.is_some() {
        info!("  Accounting period for current month already exists — skipping");
    } else {
        sqlx::query(
            r#"
            INSERT INTO accounting_periods (
                id, name, fiscal_year, starts_at, ends_at, status
            )
            VALUES ($1, $2, $3, $4, $5, 'open')
            "#,
        )
        .bind(Uuid::new_v7(Timestamp::now(NoContext)))
        .bind(&period_name)
        .bind(year)
        .bind(starts_at)
        .bind(ends_at)
        .execute(&mut **tx)
        .await?;
        info!(
            "  Accounting period: {} (fiscal year {}, {} → {})",
            period_name,
            year,
            starts_at.format("%Y-%m-%d"),
            ends_at.format("%Y-%m-%d")
        );
    }

    Ok(())
}

/// Number of days of synthetic history to generate. 120 covers two seasonal
/// cycles (m=7) so Holt-Winters has a fair shot.
const DEMAND_HISTORY_DAYS: i64 = 120;

/// Deterministic pseudo-random in [0, 1). Avoids pulling a `rand` dependency
/// for what is genuinely deterministic seed data.
fn pseudo_unit(seed: u64) -> f64 {
    // Splitmix64 — short, fast, good enough for "shake the data a bit".
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

fn dec(value: f64) -> Decimal {
    Decimal::from_f64(value)
        .unwrap_or(Decimal::ZERO)
        .round_dp(4)
}

async fn seed_demand_planning_demo(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
    cashier_id: Uuid,
) -> Result<()> {
    // ---- Category ----------------------------------------------------------
    let (cat_slug, cat_name) = data::DEMAND_DEFAULT_CATEGORY;
    let category_id: Uuid =
        match sqlx::query_as::<_, (Uuid,)>("SELECT id FROM product_categories WHERE slug = $1")
            .bind(cat_slug)
            .fetch_optional(&mut **tx)
            .await?
        {
            Some((id,)) => id,
            None => {
                let id = Uuid::new_v7(Timestamp::now(NoContext));
                sqlx::query(
                    r#"
                INSERT INTO product_categories (id, parent_id, name, slug, is_active)
                VALUES ($1, NULL, $2, $3, TRUE)
                "#,
                )
                .bind(id)
                .bind(cat_name)
                .bind(cat_slug)
                .execute(&mut **tx)
                .await?;
                id
            }
        };
    info!("  Category: {}", cat_slug);

    // ---- Vendor ------------------------------------------------------------
    let (v_code, v_name, v_legal, v_tax, v_terms) = data::DEMAND_DEFAULT_VENDOR;
    let vendor_id: Uuid =
        match sqlx::query_as::<_, (Uuid,)>("SELECT id FROM vendors WHERE code = $1")
            .bind(v_code)
            .fetch_optional(&mut **tx)
            .await?
        {
            Some((id,)) => id,
            None => {
                let id = Uuid::new_v7(Timestamp::now(NoContext));
                sqlx::query(
                    r#"
                    INSERT INTO vendors (
                        id, code, name, legal_name, tax_id,
                        payment_terms_days, currency, is_active
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, 'HNL', TRUE)
                    "#,
                )
                .bind(id)
                .bind(v_code)
                .bind(v_name)
                .bind(v_legal)
                .bind(v_tax)
                .bind(v_terms)
                .execute(&mut **tx)
                .await?;
                id
            }
        };
    info!("  Vendor: {} ({})", v_name, v_code);

    // ---- Products + stock + policies + sales history -----------------------
    let today = Utc::now().date_naive();
    let mut total_sales = 0i64;

    for (item_idx, item) in data::DEMAND_SEED_ITEMS.iter().enumerate() {
        // Product (skip insert if SKU already there).
        let product_id: Uuid =
            match sqlx::query_as::<_, (Uuid,)>("SELECT id FROM products WHERE sku = $1")
                .bind(item.sku)
                .fetch_optional(&mut **tx)
                .await?
            {
                Some((id,)) => id,
                None => {
                    let id = Uuid::new_v7(Timestamp::now(NoContext));
                    sqlx::query(
                        r#"
                        INSERT INTO products (
                            id, sku, name, category_id, unit_of_measure,
                            base_price, cost_price, currency,
                            is_perishable, is_trackable, has_variants,
                            tax_rate, tax_included, attributes, is_active
                        )
                        VALUES (
                            $1, $2, $3, $4, $5,
                            $6, $7, 'HNL',
                            FALSE, TRUE, FALSE,
                            0, FALSE, '{}'::jsonb, TRUE
                        )
                        "#,
                    )
                    .bind(id)
                    .bind(item.sku)
                    .bind(item.name)
                    .bind(category_id)
                    .bind(item.uom)
                    .bind(dec(item.base_price))
                    .bind(dec(item.cost_price))
                    .execute(&mut **tx)
                    .await?;
                    id
                }
            };

        // Stock (one row per (store, product)). Skip if already initialised.
        let stock_exists = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM inventory_stock WHERE store_id = $1 AND product_id = $2",
        )
        .bind(store_id)
        .bind(product_id)
        .fetch_optional(&mut **tx)
        .await?;
        if stock_exists.is_none() {
            sqlx::query(
                r#"
                INSERT INTO inventory_stock (
                    id, store_id, product_id, variant_id,
                    quantity, reserved_quantity, min_stock_level
                )
                VALUES ($1, $2, $3, NULL, $4, 0, $5)
                "#,
            )
            .bind(Uuid::new_v7(Timestamp::now(NoContext)))
            .bind(store_id)
            .bind(product_id)
            .bind(dec(item.on_hand_qty))
            .bind(dec(item.min_qty))
            .execute(&mut **tx)
            .await?;
        }

        // Reorder policy (one row per (variant_id, store_id) — we use
        // product_id as the "variant id" per the demand_planning convention).
        let policy_exists = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM reorder_policies WHERE product_variant_id = $1 AND store_id = $2",
        )
        .bind(product_id)
        .bind(store_id)
        .fetch_optional(&mut **tx)
        .await?;
        if policy_exists.is_none() {
            sqlx::query(
                r#"
                INSERT INTO reorder_policies (
                    id, product_variant_id, store_id,
                    min_qty, max_qty, lead_time_days,
                    safety_stock_qty, review_cycle_days,
                    preferred_vendor_id, is_active, version
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, TRUE, 0)
                "#,
            )
            .bind(Uuid::new_v7(Timestamp::now(NoContext)))
            .bind(product_id)
            .bind(store_id)
            .bind(dec(item.min_qty))
            .bind(dec(item.max_qty))
            .bind(item.lead_time_days)
            .bind(dec(item.safety_stock_qty))
            .bind(item.review_cycle_days)
            .bind(vendor_id)
            .execute(&mut **tx)
            .await?;
        }

        // ---- Synthetic 120-day sales history -------------------------------
        // Skip if we already seeded sales for this SKU.
        let already_seeded: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM sale_items
            WHERE product_id = $1 AND sku LIKE 'DP-DEMO-%'
            "#,
        )
        .bind(product_id)
        .fetch_one(&mut **tx)
        .await?;
        if already_seeded.0 > 0 {
            continue;
        }

        for d in 0..DEMAND_HISTORY_DAYS {
            // Day shifted from `today - 120` toward today.
            let day = today - Duration::days(DEMAND_HISTORY_DAYS - d);
            let weekday = day.weekday().num_days_from_monday() as f64;
            // Sinusoidal weekly pattern + small deterministic jitter in [-1, +1].
            let seasonal = (2.0 * std::f64::consts::PI * weekday / 7.0).sin();
            let seed = (item_idx as u64) * 131_071 + d as u64;
            let jitter = (pseudo_unit(seed) * 2.0) - 1.0;
            let raw = item.daily_demand_mean + item.weekly_amplitude * seasonal + jitter;
            let qty = raw.max(0.0).round();
            if qty <= 0.0 {
                continue;
            }

            let sale_id = Uuid::new_v7(Timestamp::now(NoContext));
            // Sale completed at noon local — collapse to UTC for simplicity.
            let completed_at = Utc
                .with_ymd_and_hms(day.year(), day.month(), day.day(), 18, 0, 0)
                .single()
                .unwrap_or_else(Utc::now);
            let line_total = dec(item.base_price * qty);

            sqlx::query(
                r#"
                INSERT INTO sales (
                    id, sale_number, store_id, sale_type, status,
                    cashier_id, currency,
                    subtotal, discount_value, discount_amount, tax_amount,
                    total, amount_paid, amount_due, change_given,
                    completed_at, created_at, updated_at
                )
                VALUES (
                    $1, $2, $3, 'pos', 'completed',
                    $4, 'HNL',
                    $5, 0, 0, 0,
                    $5, $5, 0, 0,
                    $6, $6, $6
                )
                "#,
            )
            .bind(sale_id)
            .bind(format!("DP-DEMO-{}-{:03}", &item.sku[3..], d))
            .bind(store_id)
            .bind(cashier_id)
            .bind(line_total)
            .bind(completed_at)
            .execute(&mut **tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO sale_items (
                    id, sale_id, line_number, product_id, variant_id,
                    sku, description, quantity, unit_of_measure,
                    unit_price, unit_cost,
                    discount_value, discount_amount, tax_rate, tax_amount,
                    subtotal, total
                )
                VALUES (
                    $1, $2, 1, $3, NULL,
                    $4, $5, $6, $7,
                    $8, $9,
                    0, 0, 0, 0,
                    $10, $10
                )
                "#,
            )
            .bind(Uuid::new_v7(Timestamp::now(NoContext)))
            .bind(sale_id)
            .bind(product_id)
            .bind(item.sku)
            .bind(item.name)
            .bind(dec(qty))
            .bind(item.uom)
            .bind(dec(item.base_price))
            .bind(dec(item.cost_price))
            .bind(line_total)
            .execute(&mut **tx)
            .await?;

            total_sales += 1;
        }
    }

    info!(
        "  Demand planning: {} products, vendor + reorder policies seeded, {} synthetic sales over {} days",
        data::DEMAND_SEED_ITEMS.len(),
        total_sales,
        DEMAND_HISTORY_DAYS
    );
    Ok(())
}

async fn seed_cash_management_defaults(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    let (bank_name, account_number, account_type, currency, opening_balance) =
        data::DEMO_BANK_ACCOUNT;
    let opening = Decimal::from_f64(opening_balance)
        .unwrap_or(Decimal::ZERO)
        .round_dp(4);

    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM bank_accounts WHERE account_number = $1")
            .bind(account_number)
            .fetch_optional(&mut **tx)
            .await?;
    if exists.is_some() {
        info!(
            "  Bank account {} already exists — skipping",
            account_number
        );
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO bank_accounts (
            id, store_id, bank_name, account_number, account_type,
            currency, current_balance, is_active, version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, TRUE, 0)
        "#,
    )
    .bind(Uuid::new_v7(Timestamp::now(NoContext)))
    .bind(store_id)
    .bind(bank_name)
    .bind(account_number)
    .bind(account_type)
    .bind(currency)
    .bind(opening)
    .execute(&mut **tx)
    .await?;

    info!(
        "  Bank account: {} {} ({}, opening L {:.2})",
        bank_name, account_number, account_type, opening_balance
    );
    Ok(())
}

async fn seed_loyalty_defaults(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    store_id: Uuid,
) -> Result<()> {
    let (program_name, program_desc, rate, expiration_days) = data::DEMO_LOYALTY_PROGRAM;

    // Program (idempotent on (store_id, name)).
    let program_id: Uuid = match sqlx::query_as::<_, (Uuid,)>(
        "SELECT id FROM loyalty_programs WHERE store_id = $1 AND name = $2",
    )
    .bind(store_id)
    .bind(program_name)
    .fetch_optional(&mut **tx)
    .await?
    {
        Some((id,)) => {
            info!(
                "  Loyalty program {} already exists — skipping",
                program_name
            );
            id
        }
        None => {
            let id = Uuid::new_v7(Timestamp::now(NoContext));
            let rate_dec = Decimal::from_f64(rate).unwrap_or(Decimal::ONE);
            sqlx::query(
                r#"
                INSERT INTO loyalty_programs (
                    id, store_id, name, description,
                    points_per_currency_unit, expiration_days, is_active
                )
                VALUES ($1, $2, $3, $4, $5, $6, TRUE)
                "#,
            )
            .bind(id)
            .bind(store_id)
            .bind(program_name)
            .bind(program_desc)
            .bind(rate_dec)
            .bind(expiration_days)
            .execute(&mut **tx)
            .await?;
            info!(
                "  Loyalty program: {} (rate {} pt/L 1, expires after {} days)",
                program_name, rate, expiration_days
            );
            id
        }
    };

    // Tiers (idempotent on (program_id, name)).
    for (name, threshold, benefits_json, sort_order) in data::DEMO_LOYALTY_TIERS {
        let exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM loyalty_member_tiers WHERE program_id = $1 AND name = $2",
        )
        .bind(program_id)
        .bind(*name)
        .fetch_optional(&mut **tx)
        .await?;
        if exists.is_some() {
            continue;
        }
        let benefits: serde_json::Value =
            serde_json::from_str(benefits_json).unwrap_or_else(|_| serde_json::json!({}));
        sqlx::query(
            r#"
            INSERT INTO loyalty_member_tiers (
                id, program_id, name, threshold_points, benefits, sort_order, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, TRUE)
            "#,
        )
        .bind(Uuid::new_v7(Timestamp::now(NoContext)))
        .bind(program_id)
        .bind(*name)
        .bind(*threshold)
        .bind(benefits)
        .bind(*sort_order)
        .execute(&mut **tx)
        .await?;
    }
    info!(
        "  Loyalty tiers: {} (Bronze/Silver/Gold)",
        data::DEMO_LOYALTY_TIERS.len()
    );

    // Rewards (idempotent — we look up by (program_id, name)).
    for reward in data::DEMO_LOYALTY_REWARDS {
        let exists: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM loyalty_rewards WHERE program_id = $1 AND name = $2")
                .bind(program_id)
                .bind(reward.name)
                .fetch_optional(&mut **tx)
                .await?;
        if exists.is_some() {
            continue;
        }
        let value_dec = Decimal::from_f64(reward.reward_value).unwrap_or(Decimal::ZERO);
        sqlx::query(
            r#"
            INSERT INTO loyalty_rewards (
                id, program_id, name, description,
                cost_points, reward_type, reward_value,
                max_redemptions_per_member, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, TRUE)
            "#,
        )
        .bind(Uuid::new_v7(Timestamp::now(NoContext)))
        .bind(program_id)
        .bind(reward.name)
        .bind(reward.description)
        .bind(reward.cost_points)
        .bind(reward.reward_type)
        .bind(value_dec)
        .bind(reward.max_per_member)
        .execute(&mut **tx)
        .await?;
    }
    info!(
        "  Loyalty rewards: {} catalog entries",
        data::DEMO_LOYALTY_REWARDS.len()
    );

    Ok(())
}
