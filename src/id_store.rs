use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
};

use crate::led::Led;
use tokio::fs;

use crate::led::Led;
use tokio::fs;

/// Represents the ID that is stored on the Tally
#[derive(PartialEq, Eq, Deserialize, Serialize, Hash, Clone, PartialOrd, Ord)]
pub struct TallyID(pub String);

impl Display for TallyID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
}

impl IDStore {
    pub fn new() -> Self {
        IDStore {
            days: HashMap::new(),
        }
    }

    /// Creats a new `IDStore` from a json file
    pub async fn new_from_json(filepath: &str) -> Result<Self, Box<dyn Error>> {
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
    pub async fn export_json(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
        fs::write(filepath, serde_json::to_string(&self)?).await?;
        Ok(())
    }

    /// Export the store to a csv file.
    /// With days in the rows and IDs in the collum.
    pub fn export_csv(&self) -> Result<String, Box<dyn Error>> {
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
        csv.push_str(&format!("ID{}{}\n", seperator, header));

        for user_id in user_ids.iter() {
            csv.push_str(&user_id.0.to_string());
            for day in days.iter() {
                let was_there: bool = self
                    .days
                    .get(day)
                    .ok_or("Failed to access day")?
                    .ids
                    .contains(user_id);

                if was_there {
                    csv.push_str(&format!("{}x", seperator));
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
        

        buzzer::beep_ack();
        led.set_named_color_time(NamedColor::Green, 1);     //led is green for 1 sec

        return true;
    }
}

fn get_day_str() -> String {
    let now = chrono::offset::Local::now();
    now.format("%Y-%m-%d").to_string()
}
