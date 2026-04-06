// PromotionRepository trait - repository for promotion operations

use async_trait::async_trait;

use crate::SalesError;
use crate::domain::entities::Promotion;
use crate::domain::value_objects::PromotionId;

/// Filter options for listing promotions
#[derive(Debug, Clone, Default)]
pub struct PromotionFilter {
    /// Filter by status (active, inactive, expired)
    pub status: Option<String>,
    /// Filter by store ID
    pub store_id: Option<uuid::Uuid>,
    /// Search by code or name
    pub search: Option<String>,
}

/// Repository trait for Promotion persistence operations.
#[async_trait]
pub trait PromotionRepository: Send + Sync {
    /// Saves a new promotion
    async fn save(&self, promotion: &Promotion) -> Result<(), SalesError>;

    /// Finds a promotion by ID
    async fn find_by_id(&self, id: PromotionId) -> Result<Option<Promotion>, SalesError>;

    /// Finds a promotion by its unique code
    async fn find_by_code(&self, code: &str) -> Result<Option<Promotion>, SalesError>;

    /// Updates an existing promotion
    async fn update(&self, promotion: &Promotion) -> Result<(), SalesError>;

    /// Finds promotions with pagination
    async fn find_paginated(
        &self,
        filter: PromotionFilter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Promotion>, i64), SalesError>;

    /// Finds all active promotions for a given store
    async fn find_active_by_store(
        &self,
        store_id: Option<uuid::Uuid>,
    ) -> Result<Vec<Promotion>, SalesError>;
}
