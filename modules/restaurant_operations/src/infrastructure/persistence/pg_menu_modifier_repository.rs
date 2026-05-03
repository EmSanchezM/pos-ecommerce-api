//! Combined Pg repository for modifier groups, modifiers, and the
//! product M2M.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::RestaurantOperationsError;
use crate::domain::entities::{MenuModifier, MenuModifierGroup};
use crate::domain::repositories::{MenuModifierRepository, ModifierGroupWithModifiers};
use crate::domain::value_objects::{MenuModifierGroupId, MenuModifierId};

pub struct PgMenuModifierRepository {
    pool: PgPool,
}

impl PgMenuModifierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MenuModifierRepository for PgMenuModifierRepository {
    async fn save_group(&self, g: &MenuModifierGroup) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO menu_modifier_groups (
                id, store_id, name, min_select, max_select,
                sort_order, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(g.id().into_uuid())
        .bind(g.store_id())
        .bind(g.name())
        .bind(g.min_select())
        .bind(g.max_select())
        .bind(g.sort_order())
        .bind(g.is_active())
        .bind(g.created_at())
        .bind(g.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_group(&self, g: &MenuModifierGroup) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE menu_modifier_groups
               SET name       = $2,
                   min_select = $3,
                   max_select = $4,
                   sort_order = $5,
                   is_active  = $6,
                   updated_at = $7
             WHERE id = $1
            "#,
        )
        .bind(g.id().into_uuid())
        .bind(g.name())
        .bind(g.min_select())
        .bind(g.max_select())
        .bind(g.sort_order())
        .bind(g.is_active())
        .bind(g.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::ModifierGroupNotFound(
                g.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_group(
        &self,
        id: MenuModifierGroupId,
    ) -> Result<Option<MenuModifierGroup>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT id, store_id, name, min_select, max_select, sort_order,
                   is_active, created_at, updated_at
            FROM menu_modifier_groups
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(MenuModifierGroup::from))
    }

    async fn list_groups_with_modifiers(
        &self,
        store_id: Uuid,
        only_active: bool,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError> {
        let groups_sql = if only_active {
            r#"
            SELECT id, store_id, name, min_select, max_select, sort_order,
                   is_active, created_at, updated_at
            FROM menu_modifier_groups
            WHERE store_id = $1 AND is_active = TRUE
            ORDER BY sort_order, name
            "#
        } else {
            r#"
            SELECT id, store_id, name, min_select, max_select, sort_order,
                   is_active, created_at, updated_at
            FROM menu_modifier_groups
            WHERE store_id = $1
            ORDER BY sort_order, name
            "#
        };
        let groups: Vec<MenuModifierGroup> = sqlx::query_as::<_, GroupRow>(groups_sql)
            .bind(store_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(MenuModifierGroup::from)
            .collect();
        if groups.is_empty() {
            return Ok(Vec::new());
        }

        let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id().into_uuid()).collect();
        let modifiers_sql = if only_active {
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE group_id = ANY($1) AND is_active = TRUE
            ORDER BY group_id, sort_order, name
            "#
        } else {
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE group_id = ANY($1)
            ORDER BY group_id, sort_order, name
            "#
        };
        let modifiers: Vec<MenuModifier> = sqlx::query_as::<_, ModifierRow>(modifiers_sql)
            .bind(&group_ids)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(MenuModifier::from)
            .collect();

        let mut bundled: Vec<ModifierGroupWithModifiers> = groups
            .into_iter()
            .map(|g| ModifierGroupWithModifiers {
                group: g,
                modifiers: Vec::new(),
            })
            .collect();
        for m in modifiers {
            if let Some(b) = bundled.iter_mut().find(|b| b.group.id() == m.group_id()) {
                b.modifiers.push(m);
            }
        }
        Ok(bundled)
    }

    async fn save_modifier(&self, m: &MenuModifier) -> Result<(), RestaurantOperationsError> {
        sqlx::query(
            r#"
            INSERT INTO menu_modifiers (
                id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(m.id().into_uuid())
        .bind(m.group_id().into_uuid())
        .bind(m.name())
        .bind(m.price_delta())
        .bind(m.sort_order())
        .bind(m.is_active())
        .bind(m.created_at())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_modifier(&self, m: &MenuModifier) -> Result<(), RestaurantOperationsError> {
        let result = sqlx::query(
            r#"
            UPDATE menu_modifiers
               SET name        = $2,
                   price_delta = $3,
                   sort_order  = $4,
                   is_active   = $5,
                   updated_at  = $6
             WHERE id = $1
            "#,
        )
        .bind(m.id().into_uuid())
        .bind(m.name())
        .bind(m.price_delta())
        .bind(m.sort_order())
        .bind(m.is_active())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(RestaurantOperationsError::ModifierNotFound(
                m.id().into_uuid(),
            ));
        }
        Ok(())
    }

    async fn find_modifier(
        &self,
        id: MenuModifierId,
    ) -> Result<Option<MenuModifier>, RestaurantOperationsError> {
        let row = sqlx::query_as::<_, ModifierRow>(
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(MenuModifier::from))
    }

    async fn list_modifiers_by_group(
        &self,
        group_id: MenuModifierGroupId,
    ) -> Result<Vec<MenuModifier>, RestaurantOperationsError> {
        let rows = sqlx::query_as::<_, ModifierRow>(
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE group_id = $1
            ORDER BY sort_order, name
            "#,
        )
        .bind(group_id.into_uuid())
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(MenuModifier::from).collect())
    }

    async fn find_modifiers_in(
        &self,
        ids: &[MenuModifierId],
    ) -> Result<Vec<MenuModifier>, RestaurantOperationsError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let raw: Vec<Uuid> = ids.iter().map(|i| i.into_uuid()).collect();
        let rows = sqlx::query_as::<_, ModifierRow>(
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE id = ANY($1)
            "#,
        )
        .bind(&raw)
        .fetch_all(&self.pool)
        .await?;
        // Reorder to match input order so the modifiers_summary matches the
        // order the cashier picked them.
        let fetched: Vec<MenuModifier> = rows.into_iter().map(MenuModifier::from).collect();
        let mut ordered = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(m) = fetched.iter().find(|m| m.id() == *id) {
                ordered.push(m.clone());
            }
        }
        Ok(ordered)
    }

    async fn assign_groups_to_product(
        &self,
        product_id: Uuid,
        group_ids: &[MenuModifierGroupId],
    ) -> Result<(), RestaurantOperationsError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM product_modifier_groups WHERE product_id = $1")
            .bind(product_id)
            .execute(&mut *tx)
            .await?;
        for gid in group_ids {
            sqlx::query(
                r#"
                INSERT INTO product_modifier_groups (product_id, group_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(product_id)
            .bind(gid.into_uuid())
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn list_groups_for_product(
        &self,
        product_id: Uuid,
    ) -> Result<Vec<ModifierGroupWithModifiers>, RestaurantOperationsError> {
        let groups: Vec<MenuModifierGroup> = sqlx::query_as::<_, GroupRow>(
            r#"
            SELECT g.id, g.store_id, g.name, g.min_select, g.max_select,
                   g.sort_order, g.is_active, g.created_at, g.updated_at
            FROM menu_modifier_groups g
            JOIN product_modifier_groups pmg ON pmg.group_id = g.id
            WHERE pmg.product_id = $1
            ORDER BY g.sort_order, g.name
            "#,
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(MenuModifierGroup::from)
        .collect();
        if groups.is_empty() {
            return Ok(Vec::new());
        }
        let group_ids: Vec<Uuid> = groups.iter().map(|g| g.id().into_uuid()).collect();
        let modifiers: Vec<MenuModifier> = sqlx::query_as::<_, ModifierRow>(
            r#"
            SELECT id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at
            FROM menu_modifiers
            WHERE group_id = ANY($1)
            ORDER BY group_id, sort_order, name
            "#,
        )
        .bind(&group_ids)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(MenuModifier::from)
        .collect();
        let mut bundled: Vec<ModifierGroupWithModifiers> = groups
            .into_iter()
            .map(|g| ModifierGroupWithModifiers {
                group: g,
                modifiers: Vec::new(),
            })
            .collect();
        for m in modifiers {
            if let Some(b) = bundled.iter_mut().find(|b| b.group.id() == m.group_id()) {
                b.modifiers.push(m);
            }
        }
        Ok(bundled)
    }
}

#[derive(sqlx::FromRow)]
struct GroupRow {
    id: Uuid,
    store_id: Uuid,
    name: String,
    min_select: i32,
    max_select: i32,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<GroupRow> for MenuModifierGroup {
    fn from(r: GroupRow) -> Self {
        MenuModifierGroup::reconstitute(
            MenuModifierGroupId::from_uuid(r.id),
            r.store_id,
            r.name,
            r.min_select,
            r.max_select,
            r.sort_order,
            r.is_active,
            r.created_at,
            r.updated_at,
        )
    }
}

#[derive(sqlx::FromRow)]
struct ModifierRow {
    id: Uuid,
    group_id: Uuid,
    name: String,
    price_delta: Decimal,
    sort_order: i32,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ModifierRow> for MenuModifier {
    fn from(r: ModifierRow) -> Self {
        MenuModifier::reconstitute(
            MenuModifierId::from_uuid(r.id),
            MenuModifierGroupId::from_uuid(r.group_id),
            r.name,
            r.price_delta,
            r.sort_order,
            r.is_active,
            r.created_at,
            r.updated_at,
        )
    }
}
