// ToggleVendorStatusUseCase - activates or deactivates a vendor

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::responses::VendorResponse;
use crate::domain::repositories::VendorRepository;
use crate::domain::value_objects::VendorId;
use crate::PurchasingError;

/// Use case for toggling vendor active status
pub struct ToggleVendorStatusUseCase<V>
where
    V: VendorRepository,
{
    vendor_repo: Arc<V>,
}

impl<V> ToggleVendorStatusUseCase<V>
where
    V: VendorRepository,
{
    /// Creates a new instance of ToggleVendorStatusUseCase
    pub fn new(vendor_repo: Arc<V>) -> Self {
        Self { vendor_repo }
    }

    /// Executes the use case to activate a vendor
    ///
    /// # Arguments
    /// * `vendor_id` - The ID of the vendor to activate
    ///
    /// # Returns
    /// VendorResponse on success
    ///
    /// # Errors
    /// * `PurchasingError::VendorNotFound` - If vendor doesn't exist
    pub async fn activate(&self, vendor_id: Uuid) -> Result<VendorResponse, PurchasingError> {
        let id = VendorId::from_uuid(vendor_id);

        let mut vendor = self
            .vendor_repo
            .find_by_id(id)
            .await?
            .ok_or(PurchasingError::VendorNotFound(vendor_id))?;

        vendor.activate();

        self.vendor_repo.update(&vendor).await?;

        Ok(self.to_response(&vendor))
    }

    /// Executes the use case to deactivate a vendor
    ///
    /// # Arguments
    /// * `vendor_id` - The ID of the vendor to deactivate
    ///
    /// # Returns
    /// VendorResponse on success
    ///
    /// # Errors
    /// * `PurchasingError::VendorNotFound` - If vendor doesn't exist
    pub async fn deactivate(&self, vendor_id: Uuid) -> Result<VendorResponse, PurchasingError> {
        let id = VendorId::from_uuid(vendor_id);

        let mut vendor = self
            .vendor_repo
            .find_by_id(id)
            .await?
            .ok_or(PurchasingError::VendorNotFound(vendor_id))?;

        vendor.deactivate();

        self.vendor_repo.update(&vendor).await?;

        Ok(self.to_response(&vendor))
    }

    fn to_response(&self, vendor: &crate::domain::entities::Vendor) -> VendorResponse {
        VendorResponse {
            id: vendor.id().into_uuid(),
            code: vendor.code().to_string(),
            name: vendor.name().to_string(),
            legal_name: vendor.legal_name().to_string(),
            tax_id: vendor.tax_id().to_string(),
            email: vendor.email().map(|s| s.to_string()),
            phone: vendor.phone().map(|s| s.to_string()),
            address: vendor.address().map(|s| s.to_string()),
            payment_terms_days: vendor.payment_terms_days(),
            currency: vendor.currency().as_str().to_string(),
            is_active: vendor.is_active(),
            notes: vendor.notes().map(|s| s.to_string()),
            created_at: vendor.created_at(),
            updated_at: vendor.updated_at(),
        }
    }
}
