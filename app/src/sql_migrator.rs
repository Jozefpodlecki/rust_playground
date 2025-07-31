use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;
use rusqlite::{Connection, OptionalExtension};
use crate::types::{LpkEntryType, LpkInfo, RunArgs};


pub fn open_sqlite_db(args: RunArgs) -> Result<()> {

    let RunArgs {
        output_path,
        ..
    } = args;

    let path = Path::new(&output_path).join("data2").join("EFTable_Ability.db");
    let connection = Connection::open(path)?;
    
     let mut stmt = connection.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name LIMIT 1"
    )?;
    
    let table_name: Option<String> = stmt.query_row([], |row| row.get(0)).optional()?;
    let table_name = table_name.unwrap();

    Ok(())
}
