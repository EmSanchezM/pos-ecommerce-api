use async_trait::async_trait;

use crate::BookingError;
use crate::domain::entities::ResourceCalendar;
use crate::domain::value_objects::ResourceId;

#[async_trait]
pub trait ResourceCalendarRepository: Send + Sync {
    /// Replace the full weekly calendar for `resource_id` with `entries` in a
    /// single transaction (delete-and-rewrite).
    async fn replace_for_resource(
        &self,
        resource_id: ResourceId,
        entries: &[ResourceCalendar],
    ) -> Result<(), BookingError>;

    async fn find_by_resource(
        &self,
        resource_id: ResourceId,
    ) -> Result<Vec<ResourceCalendar>, BookingError>;
}
