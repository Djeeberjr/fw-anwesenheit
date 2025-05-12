use std::{
    cmp::Ordering,
    fmt::Display,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

/// Represents the ID that is stored on the Tally
/// Is case-insensitive.
/// While any string can be a ID, most IDs are going to be a hex string.
#[derive(Deserialize, Serialize, Clone)]
pub struct TallyID(pub String);

impl PartialEq for TallyID {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Hash for TallyID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_uppercase().hash(state);
    }
}

impl Eq for TallyID {}

impl Ord for TallyID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.to_uppercase().cmp(&other.0.to_uppercase())
    }
}

impl PartialOrd for TallyID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for TallyID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_uppercase())
    }
}
