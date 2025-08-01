use std::{collections::HashMap, env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};

use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use duckdb::{Connection as DuckConnection, Statement};
use rusqlite::{Connection as SqliteConnection, types::Value};
use log::info;
use rusqlite::{Connection, OptionalExtension};
use crate::{lpk::LpkInfo, sql_migrator::{queries::*, sqlite_db::SqliteDb, table_schema::TableSchema, utils::*}, types::RunArgs};

pub enum MergeDirectoryFilter<'a> {
    None,
    Include(Vec<&'a str>),
    Exclude(Vec<&'a str>)
}

pub struct DbMerger {
    connection: DuckConnection,
    batch_size: usize,
}

impl DbMerger {
    pub fn new(duckdb_path: &Path, batch_size: usize) -> Result<Self> {
        let connection = DuckConnection::open(duckdb_path)?;

        Ok(Self {
            connection,
            batch_size
        })
    }

    pub fn setup(&self) -> Result<()> {
        self.connection.execute_batch(SETUP_SQL)?;
        
        Ok(())
    }

    pub fn post_work(&self) -> Result<()> {
        self.connection.execute_batch(POST_WORK_SQL)?;
        
        Ok(())
    }

    pub fn merge_directory(&self, dir: &Path, schema_name: &str, filter: &MergeDirectoryFilter) -> Result<()> {

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_path = entry.path();
            let file_name = file_path.file_name().unwrap().to_string_lossy();

            if file_path.extension().and_then(|s| s.to_str()) != Some("db") {
                continue;
            }

            let predicate = match filter {
                MergeDirectoryFilter::None => false,
                MergeDirectoryFilter::Include(items) => items.iter()
                    .find(|&&pr| file_name.contains(pr)).is_none(),
                MergeDirectoryFilter::Exclude(items) => items.iter()
                    .find(|&&pr| file_name.contains(pr)).is_some(),
            };

            if predicate {
                continue;
            }
            
            info!("Merging {}", file_name);
            self.transfer_sqlite_to_duckdb_new(&file_path, schema_name)?;
            // self.transfer_sqlite_to_duckdb(&file_path, schema_name)?;
        }
        Ok(())
    }

    fn transfer_sqlite_to_duckdb_new(&self, sqlite_path: &Path, schema_name: &str) -> Result<()> {
        let file_name = sqlite_path.file_stem().unwrap().to_string_lossy();
        let sqlite_path_str = sqlite_path.to_string_lossy().to_string();
        let connection = SqliteDb::new(sqlite_path)?;
        let table_names = connection.get_table_names()?;
        let mut query = format!("ATTACH '{}' (TYPE sqlite);\n", sqlite_path_str).to_string();

        for table_name in table_names {
            query += &format!("CREATE TABLE {}.{} AS SELECT * FROM {}.{};\n",
                schema_name, &table_name, file_name, &table_name);
        }
        
        query += &format!("DETACH {}", file_name);
        self.connection.execute_batch(&query)?;

        Ok(())
    }

    fn transfer_sqlite_to_duckdb(&self, sqlite_path: &Path, schema_name: &str) -> Result<()> {
        let connection = SqliteDb::new(sqlite_path)?;
        let table_names = connection.get_table_names()?;

        for table_name in table_names {
            let total_row_count = connection.get_row_count(&table_name)?;
            let schema = TableSchema::from_sqlite(&connection.0, &table_name)?;
            let columns: Vec<_> = schema.columns.iter()
                .map(|pr| (pr.name.as_str(), pr.mapped_type.as_str()))
                .collect();
            let create_sql = schema.to_create_table_sql(schema_name);

            self.connection.execute(&create_sql, [])?;

            let mut buffer: Vec<duckdb::types::Value> =
                Vec::with_capacity(self.batch_size * schema.columns.len());
            
            let mut row_count = 0;
            let mut row_count_it = 0;
            // let mut batch_query_cache = HashMap::new();
            let columns_length = columns.len();

            let query = &format!("SELECT * FROM {}", table_name);

            if total_row_count >= self.batch_size {
                row_count = self.batch_size;
            } else {
                row_count = total_row_count;
            }

            let placeholders = std::iter::repeat(format!("({})", vec!["?"; columns_length].join(",")))
                .take(row_count)
                .collect::<Vec<_>>()
                .join(",\n");

            let insert_sql = format!(
                "INSERT INTO {}.{} VALUES\n{}",
                schema_name, table_name, placeholders
            );

            let mut statement = self.connection.prepare(&insert_sql).unwrap();

            connection.for_each_row(query, &table_name, |row| {

                for column in schema.columns.iter() {
                    let value = row.get_ref(column.order)?;
                    let value = value_ref_to_duckdb_param(&column.col_type, value);
                    buffer.push(value);
                }

                row_count_it += 1;
                row_count += 1;

                if row_count_it >= self.batch_size {
                    print!("\r{} / {}{:10}", row_count, total_row_count, "");
             
                    statement.execute(duckdb::params_from_iter(&buffer))?;
                    buffer.clear();
                    row_count_it = 0;
                }

                Ok(())
            })?;

            if !buffer.is_empty() {
                let placeholders = std::iter::repeat(format!("({})", vec!["?"; columns_length].join(",")))
                    .take(row_count_it)
                    .collect::<Vec<_>>()
                    .join(",\n");

                let insert_sql = format!(
                    "INSERT INTO {}.{} VALUES\n{}",
                    schema_name, table_name, placeholders
                );
                self.connection.execute(&insert_sql, duckdb::params_from_iter(&buffer))?;
            }
        }

        Ok(())
    }

    // fn get_cached_insert_sql<'a>(
    //     batch_query_cache: &'a mut HashMap<usize, String>,
    //     schema_name: &str,
    //     table_name: &str,
    //     columns_length: usize,
    //     row_count: usize) -> &'a str {
    //     let insert_sql = batch_query_cache.entry(row_count)
    //         .or_insert_with(|| {
    //             let placeholders = std::iter::repeat(format!("({})", vec!["?"; columns_length].join(",")))
    //                 .take(row_count)
    //                 .collect::<Vec<_>>()
    //                 .join(",\n");

    //             let insert_sql = format!(
    //                 "INSERT INTO {}.{} VALUES\n{}",
    //                 schema_name, table_name, placeholders
    //             );

    //             insert_sql
    //         });
            

    //     insert_sql
    // }
}