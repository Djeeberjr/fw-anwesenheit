use alloc::{rc::Rc, string::String, vec::Vec};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use serde::{Deserialize, Serialize};

use crate::store::{persistence::Persistence, tally_id::TallyID};

#[derive(Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

pub struct MappingLoader<T: Persistence>(Rc<Mutex<CriticalSectionRawMutex, T>>);

impl<T: Persistence> MappingLoader<T> {
    pub fn new(persistence_layer: Rc<Mutex<CriticalSectionRawMutex, T>>) -> Self {
        Self(persistence_layer)
    }

    pub async fn get_mapping(&self, id: TallyID) -> Option<Name> {
        self.0.lock().await.load_mapping_for_id(id).await
    }

    pub async fn set_mapping(&self, id: TallyID, name: Name) {
        self.0.lock().await.save_mapping_for_id(id, name).await;
    }

    pub async fn list_mappings(&self) -> Vec<TallyID> {
        self.0.lock().await.list_mappings().await
    }
}
