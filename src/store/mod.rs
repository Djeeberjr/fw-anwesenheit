mod id_mapping;
pub mod persistence;
mod id_store;

pub use id_mapping::{IDMapping, Name};
pub use id_store::{IDStore,AttendanceDay};

pub type TallyID = [u8; 12];
pub type Date = [u8; 10];
