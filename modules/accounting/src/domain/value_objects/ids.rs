use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v7(Timestamp::now(NoContext)))
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn into_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

id_type!(AccountId);
id_type!(AccountingPeriodId);
id_type!(JournalEntryId);
id_type!(JournalLineId);
