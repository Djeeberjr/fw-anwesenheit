use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::fs;

use crate::{store::IDMapping, tally_id::TallyID};

/// Represents a single day that IDs can attend
#[derive(Deserialize, Serialize)]
pub struct AttendanceDay {
    date: String,
    ids: Vec<TallyID>,
}

/// Stores all the days
#[derive(Deserialize, Serialize)]
pub struct IDStore {
    days: HashMap<String, AttendanceDay>,
    pub mapping: IDMapping,
}

impl IDStore {
    pub fn new() -> Self {
        IDStore {
            days: HashMap::new(),
            mapping: IDMapping::new(),
        }
    }

    /// Creats a new `IDStore` from a json file
    pub async fn new_from_json(filepath: &str) -> Result<Self> {
        let read_string = fs::read_to_string(filepath).await?;
        Ok(serde_json::from_str(&read_string)?)
    }

    /// Add a new id for the current day
    /// Returns false if ID is already present at the current day.
    pub fn add_id(&mut self, id: TallyID) -> bool {
        self.get_current_day().add_id(id)
    }

    /// Get the `AttendanceDay` of the current day
    /// Creates a new if not exists
    pub fn get_current_day(&mut self) -> &mut AttendanceDay {
        let current_day = get_day_str();

        if self.days.contains_key(&current_day) {
            return self.days.get_mut(&current_day).unwrap();
        }

        self.days.insert(
            current_day.clone(),
            AttendanceDay::new(&current_day.clone()),
        );

        self.days.get_mut(&current_day.clone()).unwrap()
    }

    /// Writes the store to a json file
    pub async fn export_json(&self, filepath: &str) -> Result<()> {
        fs::write(filepath, serde_json::to_string(&self)?).await?;
        Ok(())
    }

    /// Export the store to a csv file.
    /// With days in the rows and IDs in the collum.
    pub fn export_csv(&self) -> Result<String> {
        let mut csv = String::new();
        let seperator = ";";
        let mut user_ids: HashSet<TallyID> = HashSet::new();

        for day in self.days.values() {
            for id in day.ids.iter() {
                user_ids.insert(id.clone());
            }
        }

        let mut user_ids: Vec<TallyID> = user_ids.into_iter().collect();
        user_ids.sort();

        let mut days: Vec<String> = self.days.keys().cloned().collect();
        days.sort();

        let header = days.join(seperator);
        csv.push_str(&format!(
            "ID{seperator}Nachname{seperator}Vorname{seperator}{header}\n"
        ));

        for user_id in user_ids.iter() {
            let id = &user_id.0.to_string();
            let name = self.mapping.map(user_id);

            let firstname = name.map(|e| e.first.clone()).unwrap_or("".to_owned());
            let lastname = name.map(|e| e.last.clone()).unwrap_or("".to_owned());

            csv.push_str(&format!("{id}{seperator}{lastname}{seperator}{firstname}"));
            for day in days.iter() {
                let was_there: bool = self
                    .days
                    .get(day)
                    .ok_or(anyhow!("Failed to access day"))?
                    .ids
                    .contains(user_id);

                if was_there {
                    csv.push_str(&format!("{seperator}x"));
                } else {
                    csv.push_str(seperator);
                }
            }
            csv.push('\n');
        }
        Ok(csv)
    }
}

impl AttendanceDay {
    fn new(day: &str) -> Self {
        Self {
            date: day.to_owned(),
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

fn get_day_str() -> String {
    let now = chrono::offset::Local::now();
    now.format("%Y-%m-%d").to_string()
}
