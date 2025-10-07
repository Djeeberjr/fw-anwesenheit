use super::TallyID;
use alloc::collections::BTreeMap;
use alloc::string::String;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Clone, Serialize)]
pub struct IDMapping {
    id_map: BTreeMap<TallyID, Name>,
}

impl IDMapping {
    pub fn new() -> Self {
        IDMapping {
            id_map: BTreeMap::new(),
        }
    }

    pub fn map(&self, id: &TallyID) -> Option<&Name> {
        self.id_map.get(id)
    }

    pub fn add_mapping(&mut self, id: TallyID, name: Name) {
        self.id_map.insert(id, name);
    }
}
