//! ShippingRate CRUD.

use std::str::FromStr;
use std::sync::Arc;

use uuid::Uuid;

use crate::ShippingError;
use crate::application::dtos::{
    CreateShippingRateCommand, ShippingRateResponse, UpdateShippingRateCommand,
};
use crate::domain::entities::ShippingRate;
use crate::domain::repositories::ShippingRateRepository;
use crate::domain::value_objects::{
    ShippingMethodId, ShippingRateId, ShippingRateType, ShippingZoneId,
};

pub struct CreateShippingRateUseCase {
    rate_repo: Arc<dyn ShippingRateRepository>,
}

impl CreateShippingRateUseCase {
    pub fn new(rate_repo: Arc<dyn ShippingRateRepository>) -> Self {
        Self { rate_repo }
    }

    pub async fn execute(
        &self,
        cmd: CreateShippingRateCommand,
    ) -> Result<ShippingRateResponse, ShippingError> {
        let rate_type = ShippingRateType::from_str(&cmd.rate_type)?;
        let rate = ShippingRate::create(
            ShippingMethodId::from_uuid(cmd.shipping_method_id),
            ShippingZoneId::from_uuid(cmd.shipping_zone_id),
            rate_type,
            cmd.base_rate,
            cmd.per_kg_rate,
            cmd.free_shipping_threshold,
            cmd.min_order_amount,
            cmd.max_weight_kg,
            cmd.currency,
            cmd.available_days,
            cmd.available_hour_start,
            cmd.available_hour_end,
        );
        self.rate_repo.save(&rate).await?;
        Ok(ShippingRateResponse::from(rate))
    }
}

pub struct UpdateShippingRateUseCase {
    rate_repo: Arc<dyn ShippingRateRepository>,
}

impl UpdateShippingRateUseCase {
    pub fn new(rate_repo: Arc<dyn ShippingRateRepository>) -> Self {
        Self { rate_repo }
    }

    pub async fn execute(
        &self,
        cmd: UpdateShippingRateCommand,
    ) -> Result<ShippingRateResponse, ShippingError> {
        let id = ShippingRateId::from_uuid(cmd.rate_id);
        let mut rate = self
            .rate_repo
            .find_by_id(id)
            .await?
            .ok_or(ShippingError::ShippingRateNotFound(cmd.rate_id))?;

        if let Some(b) = cmd.base_rate {
            rate.set_base_rate(b);
        }
        if let Some(p) = cmd.per_kg_rate {
            rate.set_per_kg_rate(p);
        }
        if let Some(t) = cmd.free_shipping_threshold {
            rate.set_free_threshold(t);
        }
        if let Some(active) = cmd.is_active {
            if active {
                rate.activate();
            } else {
                rate.deactivate();
            }
        }
        if cmd.available_days.is_some()
            || cmd.available_hour_start.is_some()
            || cmd.available_hour_end.is_some()
        {
            let days = cmd
                .available_days
                .unwrap_or_else(|| rate.available_days().map(<[i16]>::to_vec));
            let hs = cmd
                .available_hour_start
                .unwrap_or_else(|| rate.available_hour_start());
            let he = cmd
                .available_hour_end
                .unwrap_or_else(|| rate.available_hour_end());
            rate.set_availability(days, hs, he);
        }

        self.rate_repo.update(&rate).await?;
        Ok(ShippingRateResponse::from(rate))
    }
}

pub struct DeleteShippingRateUseCase {
    rate_repo: Arc<dyn ShippingRateRepository>,
}

impl DeleteShippingRateUseCase {
    pub fn new(rate_repo: Arc<dyn ShippingRateRepository>) -> Self {
        Self { rate_repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<(), ShippingError> {
        let rid = ShippingRateId::from_uuid(id);
        if self.rate_repo.find_by_id(rid).await?.is_none() {
            return Err(ShippingError::ShippingRateNotFound(id));
        }
        self.rate_repo.delete(rid).await
    }
}
