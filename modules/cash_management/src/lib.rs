//! # Cash Management Module
//!
//! Closes the financial loop: bank accounts, manual bank-statement entries,
//! cash deposits that link a closed `cashier_shift` to a `bank_transaction`,
//! and period-level reconciliation.
//!
//! - **Domain**: `BankAccount` (with optimistic-locking `version`),
//!   `BankTransaction`, `CashDeposit` (workflow `pending → deposited →
//!   reconciled`), `BankReconciliation` (workflow `in_progress → completed`).
//! - **Application**: use cases for CRUD + workflow transitions, and a
//!   `CashManagementEventSubscriber` that observes `cashier_shift.closed`.
//!   v1 only logs; v1.1 will auto-create a pending `CashDeposit` from the
//!   shift's cash totals.
//! - **Infrastructure**: `Pg*Repository` implementations.
//!
//! v1 does not include a `BankAdapter` registry for CSV/OFX import; manual
//! entry covers the closed loop. CSV import is bank-specific work and lives
//! in v1.1 alongside per-bank parsers.
//!
//! See `docs/roadmap-modulos.md` (Fase 1.4).

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::CashManagementError;

// Domain
pub use domain::entities::{BankAccount, BankReconciliation, BankTransaction, CashDeposit};
pub use domain::repositories::{
    BankAccountRepository, BankReconciliationRepository, BankTransactionFilter,
    BankTransactionRepository, CashDepositRepository,
};
pub use domain::value_objects::{
    BankAccountId, BankAccountType, BankReconciliationId, BankReconciliationStatus,
    BankTransactionId, BankTransactionType, CashDepositId, CashDepositStatus,
};

// Application
pub use application::dtos::{
    BankAccountResponse, BankReconciliationResponse, BankTransactionResponse, CashDepositResponse,
    CloseReconciliationCommand, CreateBankAccountCommand, CreateCashDepositCommand,
    LinkDepositCommand, MarkDepositSentCommand, RecordBankTransactionCommand,
    StartReconciliationCommand, UpdateBankAccountCommand,
};
pub use application::subscriber::CashManagementEventSubscriber;
pub use application::use_cases::{
    CloseReconciliationUseCase, CreateBankAccountUseCase, CreateCashDepositUseCase,
    DeactivateBankAccountUseCase, GetBankAccountUseCase, LinkDepositToTransactionUseCase,
    ListBankAccountsUseCase, ListBankTransactionsUseCase, ListCashDepositsUseCase,
    ListReconciliationsUseCase, MarkDepositSentUseCase, RecordBankTransactionUseCase,
    StartReconciliationUseCase, UpdateBankAccountUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgBankAccountRepository, PgBankReconciliationRepository, PgBankTransactionRepository,
    PgCashDepositRepository,
};
