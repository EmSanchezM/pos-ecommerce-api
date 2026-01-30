// UpdateVendorUseCase - updates an existing vendor

use std::sync::Arc;
use uuid::Uuid;

use crate::application::dtos::commands::UpdateVendorCommand;
use crate::application::dtos::responses::VendorResponse;
use crate::domain::repositories::VendorRepository;
use crate::domain::value_objects::VendorId;
use crate::PurchasingError;
use inventory::Currency;

/// Use case for updating an existing vendor
pub struct UpdateVendorUseCase<V>
where
    V: VendorRepository,
{
    vendor_repo: Arc<V>,
}

impl<V> UpdateVendorUseCase<V>
where
    V: VendorRepository,
{
    /// Creates a new instance of UpdateVendorUseCase
    pub fn new(vendor_repo: Arc<V>) -> Self {
        Self { vendor_repo }
    }

    /// Executes the use case to update an existing vendor
    ///
    /// # Arguments
    /// * `vendor_id` - The ID of the vendor to update
    /// * `command` - The update vendor command containing new data
    ///
    /// # Returns
    /// VendorResponse on success
    ///
    /// # Errors
    /// * `PurchasingError::VendorNotFound` - If vendor doesn't exist
    /// * `PurchasingError::DuplicateVendorTaxId` - If new tax ID already exists for another vendor
    /// * `PurchasingError::InvalidCurrency` - If currency code is invalid
    pub async fn execute(
        &self,
        vendor_id: Uuid,
        command: UpdateVendorCommand,
    ) -> Result<VendorResponse, PurchasingError> {
        let id = VendorId::from_uuid(vendor_id);

        // Find existing vendor
        let mut vendor = self
            .vendor_repo
            .find_by_id(id)
            .await?
            .ok_or(PurchasingError::VendorNotFound(vendor_id))?;

        // Check for duplicate tax ID if changing
        if let Some(ref new_tax_id) = command.tax_id
            && new_tax_id != vendor.tax_id()
                && self
                    .vendor_repo
                    .exists_by_tax_id_excluding(new_tax_id, id)
                    .await?
            {
                return Err(PurchasingError::DuplicateVendorTaxId(new_tax_id.clone()));
            }

        // Apply updates
        if let Some(name) = command.name {
            vendor.set_name(name);
        }
        if let Some(legal_name) = command.legal_name {
            vendor.set_legal_name(legal_name);
        }
        if let Some(tax_id) = command.tax_id {
            vendor.set_tax_id(tax_id);
        }
        if let Some(email) = command.email {
            vendor.set_email(Some(email));
        }
        if let Some(phone) = command.phone {
            vendor.set_phone(Some(phone));
        }
        if let Some(address) = command.address {
            vendor.set_address(Some(address));
        }
        if let Some(days) = command.payment_terms_days {
            vendor.set_payment_terms_days(days);
        }
        if let Some(currency_str) = command.currency {
            let currency = Currency::new(&currency_str)
                .map_err(|_| PurchasingError::InvalidCurrency)?;
            vendor.set_currency(currency);
        }
        if let Some(notes) = command.notes {
            vendor.set_notes(Some(notes));
        }

        // Update in repository
        self.vendor_repo.update(&vendor).await?;

        // Convert to response
        Ok(VendorResponse {
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
        })
    }
}
