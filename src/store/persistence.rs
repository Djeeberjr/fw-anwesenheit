use alloc::vec::Vec;

use crate::store::{IDMapping, day::Day, id_store::AttendanceDay};

pub trait Persistence {
    async fn load_day(&mut self, day: Day) -> Option<AttendanceDay>;
    async fn save_day(&mut self, day: Day, data: &AttendanceDay);
    async fn list_days(&mut self) -> Vec<Day>;

    async fn load_mapping(&mut self) -> Option<IDMapping>;
    async fn save_mapping(&mut self, data: &IDMapping);
}
