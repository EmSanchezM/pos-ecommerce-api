use std::sync::Arc;

use crate::TenancyError;
use crate::application::dtos::UpsertBrandingCommand;
use crate::domain::entities::OrganizationBranding;
use crate::domain::repositories::{OrganizationBrandingRepository, OrganizationRepository};
use crate::domain::value_objects::OrganizationId;

pub struct UpsertBrandingUseCase {
    orgs: Arc<dyn OrganizationRepository>,
    branding: Arc<dyn OrganizationBrandingRepository>,
}

impl UpsertBrandingUseCase {
    pub fn new(
        orgs: Arc<dyn OrganizationRepository>,
        branding: Arc<dyn OrganizationBrandingRepository>,
    ) -> Self {
        Self { orgs, branding }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
        cmd: UpsertBrandingCommand,
    ) -> Result<OrganizationBranding, TenancyError> {
        if self.orgs.find_by_id(organization_id).await?.is_none() {
            return Err(TenancyError::OrganizationNotFound(
                organization_id.into_uuid(),
            ));
        }
        let branding = OrganizationBranding::upsert_for_org(
            organization_id,
            cmd.logo_url,
            cmd.favicon_url,
            cmd.primary_color,
            cmd.secondary_color,
            cmd.accent_color,
            cmd.theme.unwrap_or_default(),
            cmd.custom_css,
        )?;
        self.branding.upsert(&branding).await?;
        Ok(branding)
    }
}

pub struct GetBrandingUseCase {
    branding: Arc<dyn OrganizationBrandingRepository>,
}

impl GetBrandingUseCase {
    pub fn new(branding: Arc<dyn OrganizationBrandingRepository>) -> Self {
        Self { branding }
    }

    pub async fn execute(
        &self,
        organization_id: OrganizationId,
    ) -> Result<OrganizationBranding, TenancyError> {
        self.branding
            .find_by_organization(organization_id)
            .await?
            .ok_or_else(|| TenancyError::BrandingNotFound(organization_id.into_uuid()))
    }
}
