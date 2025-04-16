use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
};

#[derive(PartialEq, Eq, Deserialize, Serialize, Hash, Clone, PartialOrd, Ord)]
pub struct TallyID(pub String);

#[derive(Deserialize, Serialize)]
pub struct AttendanceDay {
    date: String,
    ids: Vec<TallyID>,
}

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

    pub fn new_from_json(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let readed_string = fs::read_to_string(filepath)?;
        Ok(serde_json::from_str(&readed_string)?)
    }

    pub fn add_id(&mut self, id: TallyID) {
        let day = self.get_current_day();

        day.add_id(id);
    }

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

    pub fn export_json(&self, filepath: &str) -> Result<(), Box<dyn Error>> {
        // Serialize it to a JSON string and safe it in filepath file
        Ok(fs::write(filepath, serde_json::to_string(&self)?)?)
    }

    pub fn export_csv(&self) -> Result<String, Box<dyn Error>> {
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
        println!("ID,{}", header);

        for user_id in user_ids.iter() {
            print!("{},", user_id.0);
            for day in days.iter() {
                let was_there: bool = self.days.get(day).unwrap().ids.contains(user_id);

                if was_there {
                    print!("{}x", seperator);
                } else {
                    print!("{}", seperator);
                }
            }
            println!();
        }

        Ok("".to_owned())
    }
}

impl AttendanceDay {
    fn new(day: &str) -> Self {
        Self {
            date: day.to_owned(),
            ids: Vec::new(),
        }
    }

    fn add_id(&mut self, id: TallyID) {
        if self.ids.contains(&id) {
            return;
        }
        self.ids.push(id);
    }
}

fn get_day_str() -> String {
    let now = chrono::offset::Local::now();
    now.format("%Y-%m-%d").to_string()
}
