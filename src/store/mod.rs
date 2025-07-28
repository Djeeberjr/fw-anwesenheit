mod id_mapping;
mod id_store;

pub use id_mapping::{IDMapping, Name};
pub use id_store::IDStore;

pub type TallyID = [u8; 8];
pub type Date = u64;
