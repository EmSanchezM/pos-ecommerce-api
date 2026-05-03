//! PointsLedgerRepository — append-only ledger plus the atomic post-and-bump
//! operation. The `post_*` methods write the ledger row and update the
//! member's cached `current_balance`/`lifetime_points` in a single
//! transaction with `UPDATE ... SET balance = balance + $delta` so concurrent
//! earn/redeem calls don't lose updates.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::entities::PointsLedgerEntry;
use crate::domain::value_objects::{LoyaltyMemberId, PointsLedgerEntryId};

/// Snapshot returned by post operations so the caller has the up-to-date
/// member totals without a re-read.
#[derive(Debug, Clone)]
pub struct PostPointsResult {
    pub ledger_entry: PointsLedgerEntry,
    pub current_balance: i64,
    pub lifetime_points: i64,
}

/// One historical earn lot still on the books — emitted by
/// `find_expirable_earns` so the expiration job can fold a chunk of expired
/// lots into a single negative ledger entry.
#[derive(Debug, Clone)]
pub struct EarnedPointsLot {
    pub member_id: LoyaltyMemberId,
    pub remaining_points: i64,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait PointsLedgerRepository: Send + Sync {
    /// Append an earn entry and bump `current_balance` + `lifetime_points`
    /// atomically. Returns the post-write snapshot.
    async fn post_earn(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<PostPointsResult, LoyaltyError>;

    /// Append a redeem entry and decrement `current_balance` (lifetime untouched
    /// — it's "earn" cumulative). Fails with `InsufficientPoints` if the
    /// post-update balance would go negative.
    async fn post_redeem(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<PostPointsResult, LoyaltyError>;

    /// Append an expire entry; called by the expiration job for each member
    /// whose unredeemed earn lots have aged out.
    async fn post_expire(
        &self,
        member_id: LoyaltyMemberId,
        points: i64,
        reason: Option<String>,
    ) -> Result<PostPointsResult, LoyaltyError>;

    /// Append an admin adjustment (signed). Updates balance; lifetime is
    /// adjusted only when the delta is positive — negative adjustments don't
    /// retroactively un-earn.
    async fn post_adjustment(
        &self,
        member_id: LoyaltyMemberId,
        signed_points: i64,
        reason: String,
        created_by: Uuid,
    ) -> Result<PostPointsResult, LoyaltyError>;

    /// Returns the most recent `limit` entries for a member, newest first.
    async fn list_for_member(
        &self,
        member_id: LoyaltyMemberId,
        limit: i64,
    ) -> Result<Vec<PointsLedgerEntry>, LoyaltyError>;

    async fn find_by_id(
        &self,
        id: PointsLedgerEntryId,
    ) -> Result<Option<PointsLedgerEntry>, LoyaltyError>;

    /// Returns earn entries whose `expires_at <= now` and whose points have
    /// not yet been offset by redeems or earlier expirations. Used by the
    /// expiration job. Implementation: aggregate per member.
    async fn find_expirable_earns(
        &self,
        as_of: DateTime<Utc>,
    ) -> Result<Vec<EarnedPointsLot>, LoyaltyError>;
}
