//! TaxRate repository trait

use async_trait::async_trait;

use crate::FiscalError;
use crate::domain::entities::TaxRate;
use crate::domain::value_objects::TaxRateId;
use identity::StoreId;

/// Repository trait for TaxRate persistence
#[async_trait]
pub trait TaxRateRepository: Send + Sync {
    /// Saves a new tax rate
    async fn save(&self, tax_rate: &TaxRate) -> Result<(), FiscalError>;

    /// Finds a tax rate by ID
    async fn find_by_id(&self, id: TaxRateId) -> Result<Option<TaxRate>, FiscalError>;

    /// Finds all tax rates for a store
    async fn find_by_store(&self, store_id: StoreId) -> Result<Vec<TaxRate>, FiscalError>;

    /// Finds the default tax rate for a store
    async fn find_default(&self, store_id: StoreId) -> Result<Option<TaxRate>, FiscalError>;

    /// Finds all active tax rates for a store
    async fn find_active_by_store(&self, store_id: StoreId) -> Result<Vec<TaxRate>, FiscalError>;

    /// Updates an existing tax rate
    async fn update(&self, tax_rate: &TaxRate) -> Result<(), FiscalError>;

    /// Deletes a tax rate
    async fn delete(&self, id: TaxRateId) -> Result<(), FiscalError>;
}
