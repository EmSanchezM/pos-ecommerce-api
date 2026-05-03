//! PointsLedgerEntry — append-only audit row. The `points` field is signed:
//! `Earn`/positive `Adjustment` are positive, `Redeem`/`Expire`/negative
//! `Adjustment` are negative. Sum of all entries for a member equals their
//! current balance (the cached `LoyaltyMember.current_balance` mirrors this).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::LoyaltyError;
use crate::domain::value_objects::{LoyaltyMemberId, PointsLedgerEntryId, PointsTransactionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointsLedgerEntry {
    id: PointsLedgerEntryId,
    member_id: LoyaltyMemberId,
    txn_type: PointsTransactionType,
    /// Signed delta — negative for redeem/expire/negative-adjustment.
    points: i64,
    balance_after: i64,
    source_type: Option<String>,
    source_id: Option<Uuid>,
    occurred_at: DateTime<Utc>,
    /// Only set for `Earn` rows whose program has expiration enabled. The
    /// nightly expiration job uses this to decide what to roll into a
    /// matching `Expire` entry.
    expires_at: Option<DateTime<Utc>>,
    reason: Option<String>,
    created_by: Option<Uuid>,
}

impl PointsLedgerEntry {
    /// Constructs an `Earn` entry; `points` must be > 0.
    #[allow(clippy::too_many_arguments)]
    pub fn earn(
        member_id: LoyaltyMemberId,
        points: i64,
        balance_after: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        Ok(Self {
            id: PointsLedgerEntryId::new(),
            member_id,
            txn_type: PointsTransactionType::Earn,
            points,
            balance_after,
            source_type,
            source_id,
            occurred_at: Utc::now(),
            expires_at,
            reason,
            created_by,
        })
    }

    /// Constructs a `Redeem` entry; `points` is the positive amount being
    /// redeemed and is stored negated on the ledger.
    #[allow(clippy::too_many_arguments)]
    pub fn redeem(
        member_id: LoyaltyMemberId,
        points: i64,
        balance_after: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        Ok(Self {
            id: PointsLedgerEntryId::new(),
            member_id,
            txn_type: PointsTransactionType::Redeem,
            points: -points,
            balance_after,
            source_type,
            source_id,
            occurred_at: Utc::now(),
            expires_at: None,
            reason,
            created_by,
        })
    }

    /// Constructs an `Expire` entry; `points` is the positive amount expiring,
    /// stored negated.
    pub fn expire(
        member_id: LoyaltyMemberId,
        points: i64,
        balance_after: i64,
        reason: Option<String>,
    ) -> Result<Self, LoyaltyError> {
        if points <= 0 {
            return Err(LoyaltyError::NegativeAmount(points));
        }
        Ok(Self {
            id: PointsLedgerEntryId::new(),
            member_id,
            txn_type: PointsTransactionType::Expire,
            points: -points,
            balance_after,
            source_type: Some("expiration".into()),
            source_id: None,
            occurred_at: Utc::now(),
            expires_at: None,
            reason,
            created_by: None,
        })
    }

    /// Constructs an `Adjustment` entry; can be either positive or negative.
    /// Used for admin corrections.
    #[allow(clippy::too_many_arguments)]
    pub fn adjustment(
        member_id: LoyaltyMemberId,
        signed_points: i64,
        balance_after: i64,
        reason: String,
        created_by: Uuid,
    ) -> Result<Self, LoyaltyError> {
        if signed_points == 0 {
            return Err(LoyaltyError::NegativeAmount(0));
        }
        Ok(Self {
            id: PointsLedgerEntryId::new(),
            member_id,
            txn_type: PointsTransactionType::Adjustment,
            points: signed_points,
            balance_after,
            source_type: Some("manual".into()),
            source_id: None,
            occurred_at: Utc::now(),
            expires_at: None,
            reason: Some(reason),
            created_by: Some(created_by),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: PointsLedgerEntryId,
        member_id: LoyaltyMemberId,
        txn_type: PointsTransactionType,
        points: i64,
        balance_after: i64,
        source_type: Option<String>,
        source_id: Option<Uuid>,
        occurred_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            member_id,
            txn_type,
            points,
            balance_after,
            source_type,
            source_id,
            occurred_at,
            expires_at,
            reason,
            created_by,
        }
    }

    pub fn id(&self) -> PointsLedgerEntryId {
        self.id
    }
    pub fn member_id(&self) -> LoyaltyMemberId {
        self.member_id
    }
    pub fn txn_type(&self) -> PointsTransactionType {
        self.txn_type
    }
    pub fn points(&self) -> i64 {
        self.points
    }
    pub fn balance_after(&self) -> i64 {
        self.balance_after
    }
    pub fn source_type(&self) -> Option<&str> {
        self.source_type.as_deref()
    }
    pub fn source_id(&self) -> Option<Uuid> {
        self.source_id
    }
    pub fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
    pub fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earn_zero_is_rejected() {
        let m = LoyaltyMemberId::new();
        let err = PointsLedgerEntry::earn(m, 0, 0, None, None, None, None, None).unwrap_err();
        assert!(matches!(err, LoyaltyError::NegativeAmount(0)));
    }

    #[test]
    fn redeem_stores_negative_points() {
        let m = LoyaltyMemberId::new();
        let e = PointsLedgerEntry::redeem(m, 100, 400, None, None, None, None).unwrap();
        assert_eq!(e.points(), -100);
        assert_eq!(e.txn_type(), PointsTransactionType::Redeem);
    }

    #[test]
    fn adjustment_zero_is_rejected() {
        let m = LoyaltyMemberId::new();
        let err = PointsLedgerEntry::adjustment(
            m,
            0,
            0,
            "x".into(),
            uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)),
        )
        .unwrap_err();
        assert!(matches!(err, LoyaltyError::NegativeAmount(0)));
    }
}
