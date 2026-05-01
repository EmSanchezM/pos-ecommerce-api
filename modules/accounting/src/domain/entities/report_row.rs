use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::AccountType;

/// One row of the Profit & Loss statement: an account with its net amount in
/// the period. Convention: revenue rows are positive (credits − debits) and
/// expense rows are positive (debits − credits), so `net_amount` always reads
/// as "how much this line contributed to net income".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAndLossLine {
    pub account_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub account_type: AccountType,
    pub net_amount: Decimal,
}
