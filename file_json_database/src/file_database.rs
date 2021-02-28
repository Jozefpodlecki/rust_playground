use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::io::Read;
use std::collections::HashMap;

pub fn open_database(file_path: String) -> HashMap<String, String> {
    let mut file;

    let path = Path::new(&file_path);

    if path.exists() {
        file = fs::File::open(path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let dict: HashMap<String, String> = serde_json::from_str(&data).unwrap();
        return dict
    }

    HashMap::new()
}

pub fn save_database<T>(file_path: String, value: T) 
where
    T: Serialize {
    
    let serialized = serde_json::to_string(&value).unwrap();
    fs::write(file_path, serialized).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_() {
        
    }
}