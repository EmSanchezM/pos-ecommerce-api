// CreateVendorUseCase - creates a new vendor

use std::sync::Arc;

use crate::application::dtos::commands::CreateVendorCommand;
use crate::application::dtos::responses::VendorResponse;
use crate::domain::entities::Vendor;
use crate::domain::repositories::VendorRepository;
use crate::PurchasingError;
use inventory::Currency;

/// Use case for creating a new vendor
pub struct CreateVendorUseCase<V>
where
    V: VendorRepository,
{
    vendor_repo: Arc<V>,
}

impl<V> CreateVendorUseCase<V>
where
    V: VendorRepository,
{
    /// Creates a new instance of CreateVendorUseCase
    pub fn new(vendor_repo: Arc<V>) -> Self {
        Self { vendor_repo }
    }

    /// Executes the use case to create a new vendor
    ///
    /// # Arguments
    /// * `command` - The create vendor command containing vendor data
    ///
    /// # Returns
    /// VendorResponse on success
    ///
    /// # Errors
    /// * `PurchasingError::DuplicateVendorCode` - If vendor code already exists
    /// * `PurchasingError::DuplicateVendorTaxId` - If vendor tax ID already exists
    /// * `PurchasingError::InvalidCurrency` - If currency code is invalid
    pub async fn execute(&self, command: CreateVendorCommand) -> Result<VendorResponse, PurchasingError> {
        // Check for duplicate code
        if self.vendor_repo.exists_by_code(&command.code).await? {
            return Err(PurchasingError::DuplicateVendorCode(command.code));
        }

        // Check for duplicate tax ID
        if self.vendor_repo.exists_by_tax_id(&command.tax_id).await? {
            return Err(PurchasingError::DuplicateVendorTaxId(command.tax_id));
        }

        // Parse currency (default to HNL)
        let currency_str = command.currency.as_deref().unwrap_or("HNL");
        let currency = Currency::new(currency_str)
            .map_err(|_| PurchasingError::InvalidCurrency)?;

        // Create vendor entity
        let mut vendor = Vendor::create(
            command.code,
            command.name,
            command.legal_name,
            command.tax_id,
            currency,
        );

        // Apply optional fields
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
        if let Some(notes) = command.notes {
            vendor.set_notes(Some(notes));
        }

        // Save to repository
        self.vendor_repo.save(&vendor).await?;

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
