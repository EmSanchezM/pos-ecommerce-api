// VendorRepository trait - repository for vendor operations

use async_trait::async_trait;

use crate::domain::entities::Vendor;
use crate::domain::value_objects::VendorId;
use crate::PurchasingError;

/// Filter options for listing vendors
#[derive(Debug, Clone, Default)]
pub struct VendorFilter {
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Search by name or code
    pub search: Option<String>,
}

/// Repository trait for Vendor persistence operations.
#[async_trait]
pub trait VendorRepository: Send + Sync {
    /// Saves a new vendor to the repository
    async fn save(&self, vendor: &Vendor) -> Result<(), PurchasingError>;

    /// Finds a vendor by its unique ID
    async fn find_by_id(&self, id: VendorId) -> Result<Option<Vendor>, PurchasingError>;

    /// Finds a vendor by its code
    async fn find_by_code(&self, code: &str) -> Result<Option<Vendor>, PurchasingError>;

    /// Updates an existing vendor
    async fn update(&self, vendor: &Vendor) -> Result<(), PurchasingError>;

    /// Finds all vendors matching the filter with pagination
    ///
    /// # Arguments
    /// * `filter` - Filter options
    /// * `page` - Page number (1-indexed)
    /// * `page_size` - Number of items per page
    ///
    /// # Returns
    /// Tuple of (vendors, total_count) for pagination
    async fn find_paginated(
        &self,
        filter: VendorFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Vendor>, i64), PurchasingError>;

    /// Counts vendors matching the filter
    async fn count(&self, filter: VendorFilter) -> Result<i64, PurchasingError>;

    /// Checks if a vendor code already exists
    async fn exists_by_code(&self, code: &str) -> Result<bool, PurchasingError>;

    /// Checks if a vendor tax ID already exists
    async fn exists_by_tax_id(&self, tax_id: &str) -> Result<bool, PurchasingError>;

    /// Checks if a vendor code exists for a different vendor
    async fn exists_by_code_excluding(
        &self,
        code: &str,
        exclude_id: VendorId,
    ) -> Result<bool, PurchasingError>;

    /// Checks if a vendor tax ID exists for a different vendor
    async fn exists_by_tax_id_excluding(
        &self,
        tax_id: &str,
        exclude_id: VendorId,
    ) -> Result<bool, PurchasingError>;
}
