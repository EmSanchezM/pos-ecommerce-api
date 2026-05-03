//! # Loyalty Module
//!
//! Customer-retention building block, transversal across retail, restaurant,
//! and services verticals.
//!
//! - **Domain**: `LoyaltyProgram` (per-store config), `MemberTier`
//!   (Bronze/Silver/Gold thresholds), `LoyaltyMember` (links to a `customer`),
//!   `PointsLedgerEntry` (append-only audit trail of earn/redeem/expire),
//!   `Reward` (catalog), `RewardRedemption` (member used X points for reward Y).
//! - **Application**: program/tier/reward CRUD, `EnrollMemberUseCase`,
//!   `EarnPointsUseCase`, `RedeemRewardUseCase`, `AdjustPointsUseCase`,
//!   `ExpirePointsUseCase` (job-style), and `LoyaltyEventSubscriber` —
//!   currently logs `sale.completed` (auto-earn ships in v1.1 once sales
//!   publishes the event payload with totals).
//! - **Infrastructure**: `Pg*Repository` implementations. Member balance and
//!   lifetime totals are kept on `loyalty_members` and updated atomically
//!   alongside ledger inserts to avoid read-then-write races between
//!   concurrent earn/redeem calls.
//!
//! See `docs/roadmap-modulos.md` (Fase 2.4).

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::LoyaltyError;

// Domain
pub use domain::entities::{
    LoyaltyMember, LoyaltyProgram, MemberTier, PointsLedgerEntry, Reward, RewardRedemption,
};
pub use domain::repositories::{
    EarnedPointsLot, LoyaltyMemberRepository, LoyaltyProgramRepository, MemberTierRepository,
    PointsLedgerRepository, PostPointsResult, RewardRedemptionRepository, RewardRepository,
};
pub use domain::value_objects::{
    LoyaltyMemberId, LoyaltyProgramId, MemberTierId, PointsLedgerEntryId, PointsTransactionType,
    RewardId, RewardRedemptionId, RewardType,
};

// Application
pub use application::dtos::{
    AdjustPointsCommand, CreateLoyaltyProgramCommand, CreateMemberTierCommand, CreateRewardCommand,
    EarnPointsCommand, EnrollMemberCommand, LoyaltyMemberResponse, LoyaltyProgramResponse,
    MemberTierResponse, PointsLedgerEntryResponse, RedeemRewardCommand, RewardRedemptionResponse,
    RewardResponse,
};
pub use application::subscriber::LoyaltyEventSubscriber;
pub use application::use_cases::{
    AdjustPointsUseCase, CreateLoyaltyProgramUseCase, CreateMemberTierUseCase, CreateRewardUseCase,
    EarnPointsUseCase, EnrollMemberUseCase, ExpirePointsResult, ExpirePointsUseCase,
    GetLoyaltyMemberUseCase, GetMemberLedgerUseCase, ListLoyaltyMembersUseCase,
    ListLoyaltyProgramsUseCase, ListMemberTiersUseCase, ListRewardsUseCase, RedeemRewardUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgLoyaltyMemberRepository, PgLoyaltyProgramRepository, PgMemberTierRepository,
    PgPointsLedgerRepository, PgRewardRedemptionRepository, PgRewardRepository,
};
