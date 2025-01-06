use crate::models::MonthData;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

pub struct AppState {
    pub data: Mutex<HashMap<String, MonthData>>,
    pub data_file: String,
}

impl AppState {
    pub fn save_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.data.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data)?;
        println!("Attempting to save data to: {}", self.data_file);
        println!("Data to save: {}", json);
        fs::write(&self.data_file, &json)?;
        drop(data);
        println!("Save successful!");
        Ok(())
    }

    pub fn load_data(
        file_path: &str,
    ) -> Result<HashMap<String, MonthData>, Box<dyn std::error::Error>> {
        println!("Attempting to load data from {}", file_path);

        if !std::path::Path::new(file_path).exists() {
            println!("File not found: {}", file_path);
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(file_path)?;
        println!("File content: {}", content);
        let data: HashMap<String, MonthData> = serde_json::from_str(&content)?;
        println!("Parsed data: {:?}", data);
        Ok(data)
    }
}
