use std::collections::HashMap;

#[derive(PartialEq, Eq)]
struct TellyID (String);

struct AttendanceDay {
    date: String,
    ids: Vec<TellyID>,
}

struct IDStore {
    days: HashMap<String,AttendanceDay>
}

impl IDStore {
    fn new() -> Self {
        IDStore{
            days: HashMap::new(),
        }
    }

    fn add_id(&mut self, id: TellyID){
        let day = self.get_current_day();

        day.add_id(id);

        self.clean_map();
    }

    fn get_current_day(&mut self) -> &mut AttendanceDay {
        let current_day = get_day_str();

        if self.days.contains_key(&current_day) {
            return self.days.get_mut(&current_day).unwrap();
        }

        self.days.insert(current_day.clone(), AttendanceDay::new(&current_day.clone()));

        self.days.get_mut(&current_day.clone()).unwrap()
    }

    fn clean_map(&mut self){
       todo!()
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
