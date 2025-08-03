//! Core parsing logic for Lost Ark object files

use crate::error::ParseError;
use crate::types::{FileHeader, LostArkObject};
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde_json::{json, Value};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

pub struct Parser {
    debug: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self { debug: true }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Value> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        if self.debug {
            println!("File size: {} bytes", data.len());
            self.print_hex_preview(&data);
        }

        let mut cursor = Cursor::new(data);
        let header = self.read_header(&mut cursor)?;
        
        if self.debug {
            println!("Header: {:?}", header);
        }

        let mut object = LostArkObject::new(header);
        self.parse_properties(&mut cursor, &mut object)?;

        Ok(object.to_json())
    }

    fn print_hex_preview(&self, data: &[u8]) {
        let preview_len = std::cmp::min(64, data.len());
        print!("First {} bytes (hex): ", preview_len);
        for byte in &data[..preview_len] {
            print!("{:02x} ", byte);
        }
        println!();
    }

    fn read_header(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<FileHeader> {
        let version = cursor.read_f32::<LittleEndian>()?;
        let unknown1 = cursor.read_u32::<LittleEndian>()?;
        let object_id = cursor.read_u32::<LittleEndian>()?;
        let root_name = self.read_string(cursor)?;

        Ok(FileHeader {
            version,
            unknown1,
            object_id,
            root_name,
        })
    }

    fn read_string(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<String> {
        let length = cursor.read_u32::<LittleEndian>()?;
        
        if length > 1024 * 1024 {
            return Err(ParseError::InvalidStringLength(length).into());
        }
        
        if length == 0 {
            return Ok(String::new());
        }
        
        let mut buffer = vec![0u8; length as usize];
        cursor.read_exact(&mut buffer)?;
        
        // Remove null terminator if present
        if let Some(pos) = buffer.iter().position(|&x| x == 0) {
            buffer.truncate(pos);
        }
        
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    fn parse_properties(&self, cursor: &mut Cursor<Vec<u8>>, object: &mut LostArkObject) -> Result<()> {
        let remaining_bytes = cursor.get_ref().len() - cursor.position() as usize;
        if self.debug {
            println!("Starting property parsing with {} bytes remaining", remaining_bytes);
        }

        while cursor.position() < cursor.get_ref().len() as u64 {
            let pos_before = cursor.position();
            
            match self.read_string(cursor) {
                Ok(prop_name) => {
                    if prop_name.is_empty() || prop_name == "None" {
                        if self.debug {
                            println!("End marker found: '{}'", prop_name);
                        }
                        break;
                    }
                    
                    if self.debug {
                        println!("Reading property: '{}'", prop_name);
                    }
                    
                    // Determine property type based on name patterns
                    let value = if prop_name == "ZoneID" {
                        // Simple integer value
                        let zone_id = cursor.read_u32::<LittleEndian>()?;
                        if self.debug {
                            println!("  ZoneID value: {}", zone_id);
                        }
                        json!(zone_id)
                    } else if prop_name.ends_with("List") {
                        // Indexed property with values array
                        match self.parse_list_property(cursor) {
                            Ok(value) => value,
                            Err(e) => {
                                if self.debug {
                                    println!("Failed to parse list property {}: {}", prop_name, e);
                                }
                                self.skip_unknown_data(cursor, pos_before)?;
                                break;
                            }
                        }
                    } else {
                        // Default: try as simple value first
                        match cursor.read_u32::<LittleEndian>() {
                            Ok(simple_value) => {
                                if self.debug {
                                    println!("  Simple value: {}", simple_value);
                                }
                                json!(simple_value)
                            }
                            Err(e) => {
                                if self.debug {
                                    println!("Failed to parse as simple value: {}", e);
                                }
                                break;
                            }
                        }
                    };
                    
                    object.add_property(prop_name, value);
                }
                Err(e) => {
                    if self.debug {
                        println!("Failed to read property name at position {}: {}", pos_before, e);
                    }
                    break;
                }
            }
            
            // Safety check
            if cursor.position() == pos_before {
                if self.debug {
                    println!("No progress made, breaking loop");
                }
                break;
            }
        }
        
        Ok(())
    }

    fn parse_list_property(&self, cursor: &mut Cursor<Vec<u8>>) -> Result<Value> {
        // Show current position and next few bytes for debugging
        let pos = cursor.position();
        if self.debug {
            println!("  Current position: {}", pos);
            let remaining = cursor.get_ref().len() - pos as usize;
            if remaining >= 16 {
                let next_bytes = &cursor.get_ref()[pos as usize..pos as usize + 16];
                println!("  Next 16 bytes: {:02x?}", next_bytes);
                println!("  Looking for 10100 = 0x2774 = [74, 27, 00, 00]");
            }
        }
        
        // Read the index
        let index = cursor.read_u32::<LittleEndian>()?;
        
        if self.debug {
            println!("  List index: {}", index);
        }

        // Read the type string (like "ArrayValue")
        let type_string = self.read_string(cursor)?;
        if self.debug {
            println!("  List type: '{}'", type_string);
            
            // Show position after reading type string
            let pos_after_type = cursor.position();
            println!("  Position after type string: {}", pos_after_type);
            let remaining = cursor.get_ref().len() - pos_after_type as usize;
            if remaining >= 12 {
                let next_bytes = &cursor.get_ref()[pos_after_type as usize..pos_after_type as usize + 12];
                println!("  Next 12 bytes after type: {:02x?}", next_bytes);
            }
        }

        // For this format, there's no explicit count - just read the single value directly
        let mut values = Vec::new();
        
        // Read values based on the type - ArrayValue can contain multiple items
        if type_string == "ArrayValue" {
            // For ArrayValue, we need to read until we hit the next property or end of data
            // Keep reading values (strings or integers) until we can't read anymore valid data
            let mut item_count = 0;
            loop {
                let pos_before = cursor.position();
                
                // Try to read the next 4 bytes
                match cursor.read_u32::<LittleEndian>() {
                    Ok(potential_value) => {
                        // Check if this could be a string length
                        if potential_value > 0 && potential_value < 256 && 
                           cursor.position() + potential_value as u64 <= cursor.get_ref().len() as u64 {
                            // Reset and try reading as string
                            cursor.set_position(pos_before);
                            if let Ok(string_val) = self.read_string(cursor) {
                                // Filter out "ArrayValue" separators
                                if string_val != "ArrayValue" {
                                    if self.debug {
                                        println!("  [{}]: '{}'", item_count, string_val);
                                    }
                                    values.push(json!(string_val));
                                    item_count += 1;
                                } else if self.debug {
                                    println!("  Skipping ArrayValue separator");
                                }
                                continue;
                            }
                        }
                        
                        // Check if this could be the start of the next property (reasonable string length for property name)
                        if potential_value > 3 && potential_value < 50 {
                            // This might be the next property name length, stop reading values
                            cursor.set_position(pos_before);
                            break;
                        }
                        
                        // Otherwise treat as integer value
                        if self.debug {
                            println!("  [{}]: {}", item_count, potential_value);
                        }
                        values.push(json!(potential_value));
                        item_count += 1;
                        
                        // For single integer values like NpcIdList, stop after one
                        if item_count >= 100 || potential_value < 100000 {
                            break;
                        }
                    }
                    Err(_) => {
                        // No more data to read
                        cursor.set_position(pos_before);
                        break;
                    }
                }
            }
            
            if self.debug {
                println!("  Total items read: {}", item_count);
            }
        } else {
            // For other types, try reading multiple values with a count
            let count = cursor.read_u32::<LittleEndian>()?;
            
            if self.debug {
                println!("  List count: {}", count);
            }
            
            if count > 1000 {
                return Err(ParseError::InvalidFormat(format!("Count too large: {}", count)).into());
            }
            
            for i in 0..count {
                // Determine if this should be a string or integer based on what we can successfully read
                let pos_before = cursor.position();
            
                // Try reading as string first (check if next 4 bytes could be string length)
                if let Ok(potential_len) = cursor.read_u32::<LittleEndian>() {
                    if potential_len > 0 && potential_len < 256 && 
                       cursor.position() + potential_len as u64 <= cursor.get_ref().len() as u64 {
                        // Reset and read as string
                        cursor.set_position(pos_before);
                        if let Ok(string_val) = self.read_string(cursor) {
                            if self.debug {
                                println!("    [{}]: '{}'", i, string_val);
                            }
                            values.push(json!(string_val));
                            continue;
                        }
                    }
                    
                    // Reset and read as integer
                    cursor.set_position(pos_before);
                    if let Ok(int_val) = cursor.read_u32::<LittleEndian>() {
                        if self.debug {
                            println!("    [{}]: {}", i, int_val);
                        }
                        values.push(json!(int_val));
                    } else {
                        if self.debug {
                            println!("    [{}]: Failed to read", i);
                        }
                        break;
                    }
                } else {
                    if self.debug {
                        println!("    [{}]: No data available", i);
                    }
                    break;
                }
            }
        }
        
        Ok(json!({
            "index": index,
            "values": values
        }))
    }

    fn skip_unknown_data(&self, cursor: &mut Cursor<Vec<u8>>, start_pos: u64) -> Result<()> {
        // Reset to start position and try to find a pattern
        cursor.set_position(start_pos);
        
        // Try to find the next valid property name by looking for reasonable string lengths
        while cursor.position() < cursor.get_ref().len() as u64 - 4 {
            let pos = cursor.position();
            if let Ok(length) = cursor.read_u32::<LittleEndian>() {
                if length > 0 && length < 256 {
                    // Potential string length, try to read it
                    if cursor.position() + length as u64 <= cursor.get_ref().len() as u64 {
                        cursor.set_position(pos);
                        break;
                    }
                }
            }
            cursor.set_position(pos + 1);
        }
        
        Ok(())
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}