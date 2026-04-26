//! CalculateShippingUseCase — given a destination + order, return all
//! eligible (method, rate) options. Honors zone matching, time-of-day windows,
//! and weight/amount limits.

use std::sync::Arc;

use chrono::Utc;

use crate::ShippingError;
use crate::application::dtos::{
    CalculateShippingCommand, ShippingOptionResponse, ShippingOptionsResponse,
};
use crate::domain::repositories::{
    ShippingMethodRepository, ShippingRateRepository, ShippingZoneRepository,
};
use identity::StoreId;

pub struct CalculateShippingUseCase {
    method_repo: Arc<dyn ShippingMethodRepository>,
    zone_repo: Arc<dyn ShippingZoneRepository>,
    rate_repo: Arc<dyn ShippingRateRepository>,
}

impl CalculateShippingUseCase {
    pub fn new(
        method_repo: Arc<dyn ShippingMethodRepository>,
        zone_repo: Arc<dyn ShippingZoneRepository>,
        rate_repo: Arc<dyn ShippingRateRepository>,
    ) -> Self {
        Self {
            method_repo,
            zone_repo,
            rate_repo,
        }
    }

    pub async fn execute(
        &self,
        cmd: CalculateShippingCommand,
    ) -> Result<ShippingOptionsResponse, ShippingError> {
        let store_id = StoreId::from_uuid(cmd.store_id);

        // Find the matching zones for the destination.
        let zones = self.zone_repo.find_active_by_store(store_id).await?;
        let matching_zones: Vec<_> = zones
            .into_iter()
            .filter(|z| z.matches(&cmd.country, &cmd.state, cmd.postal_code.as_deref()))
            .collect();

        if matching_zones.is_empty() {
            return Err(ShippingError::NoMatchingZone);
        }

        let methods = self.method_repo.find_active_by_store(store_id).await?;
        let now = Utc::now();
        let mut options: Vec<ShippingOptionResponse> = Vec::new();

        for zone in &matching_zones {
            for method in &methods {
                let rates = self
                    .rate_repo
                    .find_by_method_and_zone(method.id(), zone.id())
                    .await?;
                for rate in rates.into_iter().filter(|r| {
                    r.is_active() && r.currency() == cmd.currency && r.is_available_at(now)
                }) {
                    match rate.calculate(cmd.order_total, cmd.total_weight_kg) {
                        Ok(cost) => options.push(ShippingOptionResponse {
                            method_id: method.id().into_uuid(),
                            method_code: method.code().to_string(),
                            method_name: method.name().to_string(),
                            method_type: method.method_type().to_string(),
                            zone_id: zone.id().into_uuid(),
                            zone_name: zone.name().to_string(),
                            rate_id: rate.id().into_uuid(),
                            rate: cost,
                            currency: rate.currency().to_string(),
                            estimated_days_min: method.estimated_days_min(),
                            estimated_days_max: method.estimated_days_max(),
                            is_free: cost.is_zero(),
                        }),
                        Err(_) => {
                            // Rate not eligible (weight/amount); skip.
                            continue;
                        }
                    }
                }
            }
        }

        // Cheapest first.
        options.sort_by(|a, b| a.rate.cmp(&b.rate));

        if options.is_empty() {
            return Err(ShippingError::NoRatesAvailable);
        }
        Ok(ShippingOptionsResponse { options })
    }
}
