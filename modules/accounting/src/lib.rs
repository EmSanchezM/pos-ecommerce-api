//! # Accounting Module
//!
//! General Ledger / double-entry bookkeeping for the POS + eCommerce platform.
//!
//! - **Domain**: `Account` (chart of accounts), `AccountingPeriod` (open/closed
//!   windows), `JournalEntry` + `JournalLine` (balanced debit/credit lines).
//!   The aggregate enforces sum(debits) == sum(credits) before persistence.
//! - **Application**: use cases for chart-of-accounts CRUD, period open/close,
//!   manual `PostJournalEntry`, and `GenerateProfitAndLoss`. The
//!   `AccountingEventSubscriber` observes outbox events with accounting
//!   impact (sale completed, goods receipt confirmed, payment settled,
//!   adjustment approved) — automatic posting is wired once the chart of
//!   accounts is seeded.
//! - **Infrastructure**: `Pg*Repository` implementations.
//!
//! See `docs/roadmap-modulos.md` (Fase 1.2) for the broader plan.

pub mod application;
pub mod domain;
pub mod infrastructure;

mod error;

pub use error::AccountingError;

// Domain
pub use domain::entities::{
    Account, AccountingPeriod, JournalEntry, JournalLine, ProfitAndLossLine,
};
pub use domain::repositories::{
    AccountRepository, AccountingPeriodRepository, AccountingReportRepository,
    JournalEntryRepository,
};
pub use domain::value_objects::{
    AccountId, AccountType, AccountingPeriodId, JournalEntryId, JournalEntryStatus, JournalLineId,
    PeriodStatus,
};

// Application
pub use application::dtos::{
    AccountResponse, AccountingPeriodResponse, CreateAccountCommand, JournalEntryResponse,
    JournalLineCommand, JournalLineResponse, OpenPeriodCommand, PostJournalEntryCommand,
    ProfitAndLossLineResponse, ProfitAndLossResponse,
};
pub use application::subscriber::AccountingEventSubscriber;
pub use application::use_cases::{
    ClosePeriodUseCase, CreateAccountUseCase, GenerateProfitAndLossUseCase, GetJournalEntryUseCase,
    ListAccountsUseCase, ListJournalEntriesUseCase, ListPeriodsUseCase, OpenPeriodUseCase,
    PostJournalEntryUseCase,
};

// Infrastructure
pub use infrastructure::persistence::{
    PgAccountRepository, PgAccountingPeriodRepository, PgAccountingReportRepository,
    PgJournalEntryRepository,
};
