use alloc::vec::Vec;

use crate::store::{day::Day, id_store::AttendanceDay, mapping_loader::Name, tally_id::TallyID};

pub trait Persistence {
    async fn load_day(&mut self, day: Day) -> Option<AttendanceDay>;
    async fn save_day(&mut self, day: Day, data: &AttendanceDay);
    async fn list_days(&mut self) -> Vec<Day>;

    async fn load_mapping_for_id(&mut self, id:TallyID ) -> Option<Name>;
    async fn save_mapping_for_id(&mut self, id:TallyID, name: Name);
    async fn list_mappings(&mut self) -> Vec<TallyID>;
}
