extern crate alloc;

use super::Date;
use super::IDMapping;
use super::TallyID;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

pub struct AttendanceDay {
    date: Date,
    ids: Vec<TallyID>,
}

impl AttendanceDay {
    fn new(date: Date) -> Self {
        Self {
            date,
            ids: Vec::new(),
        }
    }

    // Add an ID to the day.
    // Returns false if ID was already present
    fn add_id(&mut self, id: TallyID) -> bool {
        if self.ids.contains(&id) {
            return false;
        }
        self.ids.push(id);
        true
    }
}

pub struct IDStore {
    days: BTreeMap<Date, AttendanceDay>,
    mapping: IDMapping,
}

impl IDStore {
    pub fn new() -> Self {
        IDStore {
            days: BTreeMap::new(),
            mapping: IDMapping::new(),
        }
    }

    pub fn new_from_storage() -> Self {
        // TODO: implement
        todo!()
    }

    /// Add a new id for the current day
    /// Returns false if ID is already present at the current day.
    pub fn add_id(&mut self, id: TallyID) -> bool {
        self.get_current_day().add_id(id)
    }

    /// Get the `AttendanceDay` of the current day
    /// Creates a new if not exists
    pub fn get_current_day(&mut self) -> &mut AttendanceDay {
        let current_day: Date = 1;

        if self.days.contains_key(&current_day) {
            return self.days.get_mut(&current_day).unwrap();
        }

        self.days
            .insert(current_day, AttendanceDay::new(current_day));

        self.days.get_mut(&current_day.clone()).unwrap()
    }
}
