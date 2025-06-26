use crate::tally_id::TallyID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Name {
    pub first: String,
    pub last: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct IDMapping {
    id_map: HashMap<TallyID, Name>,
}

impl IDMapping {
    pub fn new() -> Self {
        IDMapping {
            id_map: HashMap::new(),
        }
    }

    pub fn map(&self, id: &TallyID) -> Option<&Name> {
        self.id_map.get(id)
    }

    pub fn add_mapping(&mut self, id: TallyID, name: Name) {
        self.id_map.insert(id, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut map = IDMapping::new();
        let id1 = TallyID("A2Fb44".to_owned());
        let name1 = Name {
            first: "Max".to_owned(),
            last: "Mustermann".to_owned(),
        };

        map.add_mapping(id1.clone(), name1.clone());

        let res = map.map(&id1);

        assert_eq!(res, Some(&name1));
    }

    #[test]
    fn multiple() {
        let mut map = IDMapping::new();
        let id1 = TallyID("A2Fb44".to_owned());
        let name1 = Name {
            first: "Max".to_owned(),
            last: "Mustermann".to_owned(),
        };

        let id2 = TallyID("7D3DF5B5".to_owned());
        let name2 = Name {
            first: "First".to_owned(),
            last: "Last".to_owned(),
        };

        map.add_mapping(id1.clone(), name1.clone());
        map.add_mapping(id2.clone(), name2.clone());

        let res = map.map(&id1);
        assert_eq!(res, Some(&name1));

        let res = map.map(&id2);
        assert_eq!(res, Some(&name2));
    }
}
