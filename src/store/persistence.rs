use alloc::vec::Vec;

use crate::store::{day::Day, id_store::AttendanceDay, mapping_loader::Name, tally_id::TallyID};

pub trait Persistence {
    type Error: core::error::Error;

    async fn load_day(&mut self, day: Day) -> Result<AttendanceDay, Self::Error>;
    async fn save_day(&mut self, day: Day, data: &AttendanceDay) -> Result<(), Self::Error>;
    async fn list_days(&mut self) -> Result<Vec<Day>, Self::Error>;
    async fn remove_day(&mut self, day: Day) -> Result<(),Self::Error>;

    async fn load_mapping_for_id(&mut self, id: TallyID) -> Result<Name, Self::Error>;
    async fn save_mapping_for_id(&mut self, id: TallyID, name: Name) -> Result<(), Self::Error>;
    async fn list_mappings(&mut self) -> Result<Vec<TallyID>, Self::Error>;
}
