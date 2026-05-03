use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum LoyaltyError {
    #[error("Loyalty program not found: {0}")]
    ProgramNotFound(Uuid),

    #[error("Member tier not found: {0}")]
    TierNotFound(Uuid),

    #[error("Loyalty member not found: {0}")]
    MemberNotFound(Uuid),

    #[error("Reward not found: {0}")]
    RewardNotFound(Uuid),

    #[error("Customer is already enrolled in program {program_id}")]
    AlreadyEnrolled { program_id: Uuid },

    #[error("Customer not found: {0}")]
    CustomerNotFound(Uuid),

    #[error("Reward {reward_id} belongs to a different program than member {member_id}")]
    RewardProgramMismatch { reward_id: Uuid, member_id: Uuid },

    #[error("Insufficient points: balance={balance} required={required}")]
    InsufficientPoints { balance: i64, required: i64 },

    #[error("Negative points are not allowed (got {0})")]
    NegativeAmount(i64),

    #[error("Tier threshold cannot be negative (got {0})")]
    NegativeThreshold(i64),

    #[error("Invalid points transaction type: {0}")]
    InvalidTransactionType(String),

    #[error("Invalid reward type: {0}")]
    InvalidRewardType(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Subscriber error: {0}")]
    Subscriber(String),
}
