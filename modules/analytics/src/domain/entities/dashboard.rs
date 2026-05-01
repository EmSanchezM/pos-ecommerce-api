//! Dashboard entity — a named collection of widgets owned by a user, optionally
//! scoped to a single store. The `layout` JSON describes how widgets are placed
//! in the UI grid; the API doesn't enforce a schema so the front-end can evolve
//! independently.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use uuid::Uuid;

use crate::domain::value_objects::DashboardId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    id: DashboardId,
    store_id: Option<Uuid>,
    owner_user_id: Uuid,
    name: String,
    description: Option<String>,
    layout: JsonValue,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Dashboard {
    pub fn create(
        store_id: Option<Uuid>,
        owner_user_id: Uuid,
        name: impl Into<String>,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DashboardId::new(),
            store_id,
            owner_user_id,
            name: name.into(),
            description,
            layout: json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: DashboardId,
        store_id: Option<Uuid>,
        owner_user_id: Uuid,
        name: String,
        description: Option<String>,
        layout: JsonValue,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            store_id,
            owner_user_id,
            name,
            description,
            layout,
            created_at,
            updated_at,
        }
    }

    pub fn rename(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.updated_at = Utc::now();
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_layout(&mut self, layout: JsonValue) {
        self.layout = layout;
        self.updated_at = Utc::now();
    }

    pub fn id(&self) -> DashboardId {
        self.id
    }
    pub fn store_id(&self) -> Option<Uuid> {
        self.store_id
    }
    pub fn owner_user_id(&self) -> Uuid {
        self.owner_user_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn layout(&self) -> &JsonValue {
        &self.layout
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::{NoContext, Timestamp};

    fn fresh_user_id() -> Uuid {
        Uuid::new_v7(Timestamp::now(NoContext))
    }

    #[test]
    fn create_initializes_timestamps_and_empty_layout() {
        let owner = fresh_user_id();
        let d = Dashboard::create(None, owner, "Sales overview", None);
        assert_eq!(d.name(), "Sales overview");
        assert_eq!(d.owner_user_id(), owner);
        assert_eq!(d.layout(), &json!({}));
    }

    #[test]
    fn rename_bumps_updated_at() {
        let mut d = Dashboard::create(None, fresh_user_id(), "old", None);
        let before = d.updated_at();
        std::thread::sleep(std::time::Duration::from_millis(2));
        d.rename("new");
        assert_eq!(d.name(), "new");
        assert!(d.updated_at() > before);
    }
}
