use std::{env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use duckdb::{Connection as DuckConnection};
use rusqlite::{Connection as SqliteConnection, types::Value};
use log::info;
use rusqlite::{Connection, OptionalExtension};
use crate::{lpk::LpkInfo, sql_migrator::queries::*};

pub struct SqliteDb(pub SqliteConnection);

impl SqliteDb {
    pub fn new(path: &Path) -> Result<Self> {
        let connection = Connection::open(path)?;
        Ok(Self(connection))
    }

     pub fn get_table_names(&self) -> Result<Vec<String>> {
        let mut statement = self.0.prepare(SELECT_TABLE_NAME)?;
        let table_names = statement.query_map([], |row| rusqlite::Result::Ok(row.get(0)?))
            ?.collect::<Result<_, _>>()?;
        Ok(table_names)
    }

    pub fn get_row_count(&self, table_name: &str) -> Result<usize> {
        let query = &format!("SELECT COUNT(*) FROM {}", table_name);
        let row_count = self.0.query_row(query, [], |row| row.get(0))?;
        Ok(row_count)
    }

    pub fn get_first_table_name(&self, table_name: &str) -> Result<String> {
        let table_name = self.0.query_row(SELECT_TOP_1_TABLE_NAME, [], |row| row.get(0))?;
        Ok(table_name)
    }

    pub fn get_table_schema(&self, table: &str) -> Result<Vec<(String, String)>> {
        let pragma = format!("PRAGMA table_info({})", table);
        let mut statement = self.0.prepare(&pragma)?;
        let columns = statement.query_map([], |row| {
            rusqlite::Result::Ok((row.get(1)?, row.get(2)?))
        })?.collect::<Result<_, _>>()?;
        Ok(columns)
    }
    
    pub fn for_each_row<F>(&self, query: &str, table_name: &str, mut function: F) -> Result<()>
    where
        F: FnMut(&rusqlite::Row<'_>) -> Result<()>,
    {
        let mut statement = self.0.prepare(query)?;
        let mut rows = statement.query([])?;

        while let Some(row) = rows.next()? {
            function(row)?;
        }

        Ok(())
    }

    pub fn column_count(&self, table_name: &str) -> Result<usize> {
        let query = &format!("SELECT * FROM {} LIMIT 1", table_name);
        let mut statement = self.0.prepare(query)?;
        Ok(statement.column_count())
    }
}

