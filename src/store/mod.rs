pub use id_mapping::{IDMapping, Name};
pub use id_store::{IDStore,AttendanceDay};

mod id_mapping;
pub mod persistence;
mod id_store;
pub mod tally_id;

pub type Date = [u8; 10];

