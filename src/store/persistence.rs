use alloc::vec::Vec;

use crate::store::{Date, IDMapping, id_store::AttendanceDay};

pub trait Persistence {
    async fn load_day(&mut self, day: Date) -> Option<AttendanceDay>;
    async fn save_day(&mut self, day: Date, data: &AttendanceDay);
    async fn list_days(&mut self) -> Vec<Date>;

    async fn load_mapping(&mut self) -> Option<IDMapping>;
    async fn save_mapping(&mut self, data: &IDMapping);
}
