//! PgPointsLedgerRepository — every `post_*` method runs the ledger insert
//! and the cached-totals UPDATE inside a single transaction. The UPDATE on
//! `loyalty_members` uses `balance = balance + $delta` so concurrent earn or
//! redeem requests can't lose updates to a read-then-write race.
//!
//! Redeem additionally checks the post-update balance and rolls back if it
//! went negative — the caller surfaces this as `InsufficientPoints`. The
//! transaction-level guard is the source of truth even though the use case
//! also pre-checks: another concurrent redeem could win between read and
//! UPDATE, and we don't want to allow over-redemption.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::PointsLedgerEntry;
use crate::domain::repositories::{EarnedPointsLot, PointsLedgerRepository, PostPointsResult};
use crate::domain::value_objects::{LoyaltyMemberId, PointsLedgerEntryId, PointsTransactionType};

pub struct PgPointsLedgerRepository {
    pool: PgPool,
}

impl PgPointsLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Shared "insert ledger row + bump member totals" body. `lifetime_delta`
    /// is added to `lifetime_points`; pass 0 for redeem/expire/negative
    /// adjustments. Returns the post-update totals.
    #[allow(clippy::too_many_arguments)]
    async fn post_internal(
        &self,
        member_id: LoyaltyMemberId,
        txn_type: PointsTransactionType,
        signed_points: i64,
        lifetime_delta: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<PostPointsResult, LoyaltyError> {
        let mut tx = self.pool.begin().await?;

        // Atomic balance bump. If the row doesn't exist we get None and bail;
        // if the post-update balance went negative we abort so redeems never
        // overdraft.
        let totals: Option<(i64, i64)> = sqlx::query_as(
            r#"
            UPDATE loyalty_members
            SET current_balance  = current_balance + $2,
                lifetime_points  = lifetime_points + $3,
                updated_at       = NOW()
            WHERE id = $1
            RETURNING current_balance, lifetime_points
            "#,
        )
        .bind(member_id.into_uuid())
        .bind(signed_points)
        .bind(lifetime_delta)
        .fetch_optional(&mut *tx)
        .await?;

        let (current_balance, lifetime_points) =
            totals.ok_or_else(|| LoyaltyError::MemberNotFound(member_id.into_uuid()))?;

        if current_balance < 0 {
            // Roll back; the caller will see InsufficientPoints (or their own
            // pre-check, which surfaces a clearer error).
            tx.rollback().await?;
            return Err(LoyaltyError::InsufficientPoints {
                balance: current_balance - signed_points,
                required: -signed_points,
            });
        }

        let entry_id = PointsLedgerEntryId::new();
        let occurred_at = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO loyalty_points_ledger (
                id, member_id, txn_type, points, balance_after,
                source_type, source_id, occurred_at, expires_at,
                reason, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(entry_id.into_uuid())
        .bind(member_id.into_uuid())
        .bind(txn_type.to_string())
        .bind(signed_points)
        .bind(current_balance)
        .bind(source_type.as_deref())
        .bind(source_id)
        .bind(occurred_at)
        .bind(expires_at)
        .bind(reason.as_deref())
        .bind(created_by)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let entry = PointsLedgerEntry::reconstitute(
            entry_id,
            member_id,
            txn_type,
            signed_points,
            current_balance,
            source_type,
            source_id,
            occurred_at,
            expires_at,
            reason,
            created_by,
        );

        Ok(PostPointsResult {
            ledger_entry: entry,
            current_balance,
            lifetime_points,
        })
    }
}

#[async_trait]
impl PointsLedgerRepository for PgPointsLedgerRepository {
    async fn post_earn(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        self.post_internal(
            member_id,
            PointsTransactionType::Earn,
            points,
            points,
            source_type,
            source_id,
            expires_at,
            reason,
            created_by,
        )
        .await
    }

    async fn post_redeem(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        self.post_internal(
            member_id,
            PointsTransactionType::Redeem,
            -points,
            0,
            source_type,
            source_id,
            None,
            reason,
            created_by,
        )
        .await
    }

    async fn post_expire(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        reason: Option<String>,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        self.post_internal(
            member_id,
            PointsTransactionType::Expire,
            -points,
            0,
            Some("expiration".into()),
            None,
            None,
            reason,
            None,
        )
        .await
    }

    async fn post_adjustment(
        &self,
        member_id: LoyaltyMemberId,
        signed_points: i64,
        reason: String,
        created_by: Uuid,
    ) -> Result<PostPointsResult, LoyaltyError> {
        if signed_points == 0 {
            return Err(LoyaltyError::NegativeAmount(0));
        }
        // Positive admin adjustments count toward lifetime; negatives don't
        // un-earn.
        let lifetime_delta = if signed_points > 0 { signed_points } else { 0 };
        self.post_internal(
            member_id,
            PointsTransactionType::Adjustment,
            signed_points,
            lifetime_delta,
            Some("manual".into()),
            None,
            None,
            Some(reason),
            Some(created_by),
        )
        .await
    }

    async fn list_for_member(
        &self,
        member_id: LoyaltyMemberId,
        limit: i64,
    ) -> Result<Vec<PointsLedgerEntry>, LoyaltyError> {
        let rows = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT id, member_id, txn_type, points, balance_after,
                   source_type, source_id, occurred_at, expires_at,
                   reason, created_by
            FROM loyalty_points_ledger
            WHERE member_id = $1
            ORDER BY occurred_at DESC, id DESC
            LIMIT $2
            "#,
        )
        .bind(member_id.into_uuid())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(PointsLedgerEntry::try_from).collect()
    }

    async fn find_by_id(
        &self,
        id: PointsLedgerEntryId,
    ) -> Result<Option<PointsLedgerEntry>, LoyaltyError> {
        let row = sqlx::query_as::<_, LedgerRow>(
            r#"
            SELECT id, member_id, txn_type, points, balance_after,
                   source_type, source_id, occurred_at, expires_at,
                   reason, created_by
            FROM loyalty_points_ledger
            WHERE id = $1
            "#,
        )
        .bind(id.into_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(PointsLedgerEntry::try_from).transpose()
    }

    async fn find_expirable_earns(
        &self,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<EarnedPointsLot>, LoyaltyError> {
        // Per-member: take the sum of earns whose expires_at <= as_of and
        // subtract any expire/redeem entries we've already posted since the
        // earn boundary. We use the cached `current_balance` as a ceiling so
        // we don't try to expire more than the member actually has.
        //
        // The query returns one row per member with an expirable surplus.
        // `expires_at` returned is the latest in the bucket (informational).
        let rows = sqlx::query_as::<_, ExpirableRow>(
            r#"
            WITH expired_earns AS (
                SELECT member_id,
                       SUM(points) AS expired_points,
                       MAX(expires_at) AS latest_expires_at
                FROM loyalty_points_ledger
                WHERE txn_type = 'earn'
                  AND expires_at IS NOT NULL
                  AND expires_at <= $1
                GROUP BY member_id
            ),
            already_expired AS (
                SELECT member_id, COALESCE(SUM(-points), 0) AS already_expired_points
                FROM loyalty_points_ledger
                WHERE txn_type = 'expire'
                GROUP BY member_id
            )
            SELECT m.id AS member_id,
                   GREATEST(
                       0,
                       LEAST(
                           e.expired_points - COALESCE(a.already_expired_points, 0),
                           m.current_balance
                       )
                   )::BIGINT AS remaining,
                   e.latest_expires_at AS expires_at
            FROM loyalty_members m
            JOIN expired_earns e ON e.member_id = m.id
            LEFT JOIN already_expired a ON a.member_id = m.id
            WHERE m.current_balance > 0
            "#,
        )
        .bind(as_of)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .filter(|r| r.remaining > 0)
            .map(|r| EarnedPointsLot {
                member_id: LoyaltyMemberId::from_uuid(r.member_id),
                remaining_points: r.remaining,
                expires_at: r.expires_at,
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct LedgerRow {
    id: Uuid,
    member_id: Uuid,
    txn_type: String,
    points: i64,
    balance_after: i64,
    source_type: Option<String>,
    source_id: Option<Uuid>,
    occurred_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    reason: Option<String>,
    created_by: Option<Uuid>,
}

impl TryFrom<LedgerRow> for PointsLedgerEntry {
    type Error = LoyaltyError;

    fn try_from(row: LedgerRow) -> Result<Self, Self::Error> {
        let t = PointsTransactionType::from_str(&row.txn_type)?;
        Ok(PointsLedgerEntry::reconstitute(
            PointsLedgerEntryId::from_uuid(row.id),
            LoyaltyMemberId::from_uuid(row.member_id),
            t,
            row.points,
            row.balance_after,
            row.source_type,
            row.source_id,
            row.occurred_at,
            row.expires_at,
            row.reason,
            row.created_by,
        ))
    }
}

#[derive(sqlx::FromRow)]
struct ExpirableRow {
    member_id: Uuid,
    remaining: i64,
    expires_at: DateTime<Utc>,
}
