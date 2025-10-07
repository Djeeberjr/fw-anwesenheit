use alloc::vec::Vec;
use serde::Deserialize;
use serde::Serialize;

use super::Date;
use super::IDMapping;
use super::TallyID;
use crate::store::persistence::Persistence;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AttendanceDay {
    date: Date,
    ids: Vec<TallyID>,
}

impl AttendanceDay {
    pub fn new(date: Date) -> Self {
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

#[derive(Clone)]
pub struct IDStore<T: Persistence> {
    pub current_day: AttendanceDay,
    pub mapping: IDMapping,
    persistence_layer: T,
}

impl<T: Persistence> IDStore<T> {
    pub async fn new_from_storage(mut persistence_layer: T) -> Self {
        // let mapping = match persistence_layer.load_mapping().await {
        //     Some(map) => map,
        //     None => IDMapping::new(),
        // };

        let current_date: Date = [0; 10];

        let day = persistence_layer
            .load_day(current_date)
            .await
            .unwrap_or(AttendanceDay::new(current_date));

        Self {
            current_day: day,
            mapping: IDMapping::new(),
            persistence_layer,
        }
    }

    async fn persist_day(&mut self) {
        self.persistence_layer
            .save_day(self.current_day.date, &self.current_day)
            .await
    }

    async fn persist_mapping(&mut self) {
        self.persistence_layer.save_mapping(&self.mapping).await
    }

    /// Add a new id for the current day
    /// Returns false if ID is already present at the current day.
    pub async fn add_id(&mut self, id: TallyID, current_date: Date) -> bool {
        if self.current_day.date == current_date {
            let changed = self.current_day.add_id(id);
            if changed {
                self.persist_day().await;
            }
            return changed;
        }

        let new_day = AttendanceDay::new(current_date);
        self.current_day = new_day;

        let changed = self.current_day.add_id(id);
        if changed {
            self.persist_day().await;
        }
        changed
    }
}
