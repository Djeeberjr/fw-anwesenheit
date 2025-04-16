use std::{collections::HashMap, error::Error, fs::{self, read_to_string}, result};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq)]
#[derive(Deserialize, Serialize)]
struct TellyID (String);

#[derive(Deserialize, Serialize)]
struct AttendanceDay {
    date: String,
    ids: Vec<TellyID>,
}

#[derive(Deserialize, Serialize)]
struct IDStore {
    days: HashMap<String,AttendanceDay>
}

impl IDStore {
    
    fn new() -> Self {
        IDStore{
            days: HashMap::new(),
        }
    }

    fn new_from_json(filepath:&str) -> Result<Self, Box<dyn Error>>{
        let readed_string = fs::read_to_string(filepath)?;
        Ok(serde_json::from_str(&readed_string)?)
    }

    fn add_id(&mut self, id: TellyID){
        let day = self.get_current_day();

        day.add_id(id);
    
    }

    fn get_current_day(&mut self) -> &mut AttendanceDay {
        let current_day = get_day_str();

        if self.days.contains_key(&current_day) {
            return self.days.get_mut(&current_day).unwrap();
        }

        self.days.insert(current_day.clone(), AttendanceDay::new(&current_day.clone()));

        self.days.get_mut(&current_day.clone()).unwrap()
    }

    fn export_jason(&self, filepath:&str) -> Result <(), Box<dyn Error>> {

        // Serialize it to a JSON string and safe it in filepath file
        Ok(fs::write("attendence_list.json", serde_json::to_string(&self)?)?)
    }
}

impl AttendanceDay {
    fn new(day: &str) -> Self{
        Self{
            date: day.to_owned(),
            ids: Vec::new(),
        }
    }

    fn add_id(&mut self, id: TellyID){
        if self.ids.contains(&id) {
            return
        } 
        self.ids.push(id);
    }
}

fn get_day_str() -> String {
    let now = chrono::offset::Local::now();
    now.format("%Y-%m-%d").to_string()
}
