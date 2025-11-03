use alloc::rc::Rc;
use alloc::vec::Vec;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use serde::Deserialize;
use serde::Serialize;

use crate::store::day::Day;
use crate::store::persistence::Persistence;
use crate::store::tally_id::TallyID;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AttendanceDay {
    date: Day,
    ids: Vec<TallyID>,
}

impl AttendanceDay {
    pub fn new(date: Day) -> Self {
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
    current_day: AttendanceDay,
    persistence_layer: Rc<Mutex<CriticalSectionRawMutex, T>>,
}

impl<T: Persistence> IDStore<T> {
    pub async fn new_from_storage(
        persistence_layer: Rc<Mutex<CriticalSectionRawMutex, T>>,
        current_date: Day,
    ) -> Self {
        let day = persistence_layer
            .lock()
            .await
            .load_day(current_date)
            .await
            .unwrap_or(AttendanceDay::new(current_date));

        Self {
            current_day: day,
            persistence_layer,
        }
    }

    async fn persist_day(&mut self) -> Result<(), T::Error> {
        self.persistence_layer
            .lock()
            .await
            .save_day(self.current_day.date, &self.current_day)
            .await
    }

    /// Add a new id for the current day
    /// Returns false if ID is already present at the current day.
    pub async fn add_id(&mut self, id: TallyID, current_date: Day) -> bool {
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

    /// Load and return a AttendanceDay. Nothing more. Nothing less.
    pub async fn load_day(&mut self, day: Day) -> Result<AttendanceDay, T::Error> {
        if day == self.current_day.date {
            return Ok(self.current_day.clone());
        }

        self.persistence_layer.lock().await.load_day(day).await
    }

    pub async fn list_days_in_timespan(
        &mut self,
        from: Day,
        to: Day,
    ) -> Result<Vec<Day>, T::Error> {
        let all_days = self.persistence_layer.lock().await.list_days().await?;

        Ok(all_days
            .into_iter()
            .filter(|e| *e >= from)
            .filter(|e| *e <= to)
            .collect())
    }
}
