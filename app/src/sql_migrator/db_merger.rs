use std::{collections::HashMap, env, fs::{self, File}, io::{BufWriter, Cursor, Read, Seek, Write}, path::{Path, PathBuf}};
use strum_macros::{VariantArray, VariantNames};
use strum_macros::{AsRefStr, EnumString};
use anyhow::*;
use byteorder::{LittleEndian, ReadBytesExt};
use duckdb::{params, Connection as DuckConnection, Statement};
use rusqlite::{Connection as SqliteConnection, types::Value};
use log::info;
use rusqlite::{Connection, OptionalExtension};
use walkdir::WalkDir;
use crate::{enum_extractor::extract_enum_maps_from_xml, loa_extractor::collect_loa_files, lpk::{self, get_lpks, LpkInfo}, process_dumper::ProcessDumper, sql_migrator::{queries::*, sqlite_db::SqliteDb, table_schema::TableSchema, types::ColumnAction, utils::*, DuckDb}, types::{RunArgs, WaitStrategy}};
use capstone::{arch::{self, BuildsCapstone, BuildsCapstoneSyntax}, Capstone};

#[derive(Debug, Default)]
pub enum TransformationStrategy {
    #[default]
    AttachSqlite,
    BatchInsert
}

#[derive(Debug)]
pub struct DbFileEntry {
    pub file_path: PathBuf,
    pub file_name: String,
    pub strategy: TransformationStrategy,
}

pub struct DbMerger {
    connection: DuckDb,
    batch_size: usize,
}

impl DbMerger {
    pub fn new(duckdb_path: &Path, batch_size: usize) -> Result<Self> {
        let connection = DuckDb::new(duckdb_path)?;
        // connection.execute("SET default_block_size = 32768;", [])?;

        Ok(Self {
            connection,
            batch_size
        })
    }

    pub fn setup(&self) -> Result<()> {
        let script = include_str!("./scripts/before_migration.sql");
        self.connection.execute_batch(script);
        
        Ok(())
    }

    pub fn post_work(&self) -> Result<String> {
        let script = include_str!("./scripts/post_migration.sql");
        self.connection.execute_batch(script);

        let mut result = String::from("");

        info!("generate_update_localization_script");
        let script = &self.connection.generate_update_localization_script()?;
        self.connection.execute_batch(script);
        result += script;

        let test = self.connection.generate_rust_struct("data", "Ability")?;
        let mut file = File::create("models.rs")?;
        file.write_all(test.as_bytes())?;

        let script = include_str!("./scripts/post_migration.sql");
        self.connection.execute_batch(script);
        result += script;

        info!("generate_drop_empty_columns");
        let comment_column_map = self.connection.get_column_values("Comment")?;
        let script = &self.connection.generate_drop_empty_columns_script("Comment", comment_column_map);
        self.connection.execute_batch(script);
        result += script;

        info!("drop_empty_tables");
        let script = &self.connection.generate_drop_empty_tables_script()?;
        self.connection.execute_batch(script);
        result += script;
        
        info!("drop_unused_secondary_keys");
        let script = &self.connection.generate_drop_unused_secondary_keys_script("SecondaryKey")?;
        self.connection.execute_batch(script);
        result += script;

        info!("rename PrimaryKey");
        let script = &self.connection.generate_column_script("PrimaryKey", ColumnAction::Rename("Id".to_string()))?;
        self.connection.execute_batch(script);
        result += script;

        info!("rename SecondaryKey");
        let script = &self.connection.generate_column_script("SecondaryKey", ColumnAction::Rename("SubId".to_string()))?;
        self.connection.execute_batch(script);
        result += script;

        info!("rename Desc");
        let script = &self.connection.generate_column_script("Desc", ColumnAction::Rename("Description".to_string()))?;
        self.connection.execute_batch(script);
        result += script;

        info!("drop Milestone");
        let script = &self.connection.generate_column_script("Milestone", ColumnAction::Drop)?;
        self.connection.execute_batch(script);
        result += script;

        info!("drop SourceRow");
        let script = &self.connection.generate_column_script("SourceRow", ColumnAction::Drop)?;
        self.connection.execute_batch(script);
        result += script;

        info!("integer_downgrade");
        let script = &self.connection.generate_integer_downgrade_script()?;
        self.connection.execute_batch(script);
        result += script;

        info!("primary_keys");
        let script = &self.connection.generate_primary_keys_script()?;
        self.connection.execute_batch(script);
        result += script;

        info!("primary_keys");
        let script = &self.connection.generate_global_search_view()?;
        self.connection.execute_batch(script);
        result += script;

        self.connection.execute_batch("CHECKPOINT;")?;
        
        Ok(result)
    }

    pub fn merge_data(&self, sqlite_dir: PathBuf) -> Result<()> {
        let mut entries = collect_db_file_entries(&sqlite_dir)?;
        let entry = entries.get_mut("EFTable_LanguageGrams").unwrap();
        entry.strategy = TransformationStrategy::BatchInsert;
        self.merge(entries, "data");
        Ok(())
    }

    pub fn merge_jss(&self, sqlite_dir: PathBuf) -> Result<()> {
        let mut entries: HashMap<String, DbFileEntry> = collect_db_file_entries(&sqlite_dir)?;
        self.merge(entries, "jss");

        Ok(())
    }

    pub fn merge(&self, entries: HashMap<String, DbFileEntry>, schema_name: &str) -> Result<()> {

        for (_, entry) in entries {
         
            let file_path = entry.file_path;
            info!("Merging {}", entry.file_name);

            match entry.strategy {
                TransformationStrategy::AttachSqlite => self.transfer_sqlite_to_duckdb_by_attach(&file_path, schema_name)?,
                TransformationStrategy::BatchInsert => self.transfer_sqlite_to_duckdb_by_insert(&file_path, schema_name)?,
            };
        }

        Ok(())
    }

    pub fn insert_lpk_metadata(&self,
        lpk_path: &Path,
        cipher_key: Vec<u8>,
        aes_xor_key: Vec<u8>,
    ) -> Result<()> {

        let lpks = get_lpks(
            &lpk_path,
            &cipher_key,
            &aes_xor_key)?;

        for lpk in lpks {

            let full_table_name = lpk.name.clone();

            let query = format!(r"
                CREATE TABLE lpk.{}
                (
                    Id INT NOT NULL PRIMARY KEY,
                    Name VARCHAR(50) NOT NULL,
                    FilePath VARCHAR(100) NOT NULL,
                    Extension VARCHAR(4) NOT NULL,
                    Size INT NOT NULL
                );
            ", &full_table_name);

            info!("Creating table {}", full_table_name);
            self.connection.execute_batch(&query)?;

            for chunk in lpk.get_summary().chunks(1000) {
                
                 let placeholders = std::iter::repeat("(?, ?, ?, ?, ?)")
                    .take(chunk.len())
                    .collect::<Vec<_>>()
                    .join(",\n");

                let insert_sql = format!(
                    r#"
                    INSERT INTO lpk."{}" (Id, Name, FilePath, Extension, Size)
                    VALUES
                    {};
                    "#,
                    full_table_name,
                    placeholders
                );

                let mut params = Vec::with_capacity(chunk.len() * 5);
                for entry in chunk {
                    params.extend([
                        duckdb::types::Value::Int(entry.order as i32),
                        duckdb::types::Value::Text(entry.file_name.clone()),
                        duckdb::types::Value::Text(entry.file_path.clone()),
                        duckdb::types::Value::Text(entry.content_type.as_ref().to_string()),
                        duckdb::types::Value::Int(entry.max_length as i32),
                    ]);
                }

                self.connection.execute(&insert_sql, duckdb::params_from_iter(params))?;
            }
        }

        Ok(())
    }

    pub fn insert_assembly(
        &self,
        exe_path: PathBuf,
        dest_path: &Path
    ) -> Result<()> {

        let mut process_dumper = ProcessDumper::new(&exe_path, dest_path)?;

        let mut cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .build()?;

        cs.set_skipdata(true)?;
        cs.set_detail(true)?;

        let blocks = process_dumper.get_cached()?;

        for block in blocks {

            let module = block.block.module.as_ref();
            let mut data = vec![];

            if (module.filter(|pr| pr.file_name == "LOSTARK.exe").is_none()
                && module.filter(|pr| pr.file_name == "EFEngine.dll").is_none()) {
                continue;
            } else {
                data = process_dumper.read_block_data(&block)?;
            }

            let block = block.block;

            let table_name = block.module.map(|pr| format!("{}_0x{:X}", pr.file_name, block.base))
                .unwrap_or_else(|| format!("{}_0x{:X}", "unknown", block.base))
                .replace(".", "_");

            if block.is_executable {
                let table_name = format!("assembly.{}_text", table_name);
                info!("Creating {}", table_name);
                let query: String = format!(r"
                    CREATE TABLE {}
                    (
                        Id BIGINT NOT NULL PRIMARY KEY, 
                        Mnemonic VARCHAR(20) NOT NULL,
                        OpStr VARCHAR NOT NULL

                    );", table_name);

                self.connection.execute_batch(&query)?;
                let query = format!("INSERT INTO {} (Id, Mnemonic, OpStr) VALUES (?, ?, ?)", table_name);
                let mut statement = self.connection.prepare(&query)?;

                let instructions = cs.disasm_all(&data, 0)?;

                for instruction in instructions.into_iter() {
                    let address = instruction.address() as i64;
                    let mnemonic = instruction.mnemonic().unwrap_or("");
                    let op_str = instruction.op_str().unwrap_or("");


                    statement.execute(params![address, mnemonic, op_str])?;
                }
            }

        }

        Ok(())
    }

    pub fn insert_loa_data(&self, output_path: &Path) -> Result<()> {
        const TABLE_NAME: &str = "lpk.LoaFiles";

        let query = r"
        CREATE TABLE lpk.LoaFiles
        (
            Id INT NOT NULL PRIMARY KEY,
            FilePath VARCHAR(100) NOT NULL,
            Name VARCHAR(100) NOT NULL,
            Data VARCHAR NOT NULL
        );";

        self.connection.execute_batch(query)?;
        let mut buffer = Vec::with_capacity(self.batch_size * 2);
        let mut count = 0;
        let files = collect_loa_files(output_path)?;

        for entry in files {
            info!("Extracting data from {}", entry.relative_path);

            buffer.push(duckdb::types::Value::Int(entry.id));
            buffer.push(duckdb::types::Value::Text(entry.relative_path));
            buffer.push(duckdb::types::Value::Text(entry.name));
            let keywords: String = entry.keywords.join(",");
            buffer.push(duckdb::types::Value::Text(keywords));

            count += 1;

            if count % self.batch_size == 0 {
                let placeholders = std::iter::repeat("(?, ?, ?, ?)")
                    .take(self.batch_size)
                    .collect::<Vec<_>>()
                    .join(",\n");

                let insert_sql = format!(
                    "INSERT INTO {} (Id, FilePath, Name, Data)\nVALUES\n{};",
                    TABLE_NAME, placeholders
                );

                self.connection.execute(&insert_sql, duckdb::params_from_iter(&buffer))?;
                buffer.clear();
            }
        }

        Ok(())
    }

    pub fn create_enums(&self, lpk_path: &Path) -> Result<()> {
        let schema_name = "enums";
        
        if self.connection.tables_exists(schema_name)? {
            info!("Skipping creating enums");
            return Ok(());
        }
        
        let enum_path = lpk_path.join(r"data1\Common\StringData\EFGameMsg_Enums.xml");
        let enums = extract_enum_maps_from_xml(&enum_path)?;

        for (enum_name, entries) in enums {
            let table_name = enum_name.replace("_", "");
            let full_table_name = format!("{}.{}", schema_name, table_name);

            let max_id = *entries.keys().max().unwrap_or(&0);
            let id_type = get_duckdb_int_type_for_enum_keys(max_id);

            let create_sql = format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    Id {} PRIMARY KEY,
                    Name VARCHAR(20) NOT NULL
                );",
                full_table_name, id_type
            );

            info!("Creating table {}", full_table_name);
            self.connection.execute_batch(&create_sql)?;

            let index_sql = format!("CREATE INDEX IF NOT EXISTS IX_ENUM_{}_Name ON {} (Name);", table_name, full_table_name);
            self.connection.execute_batch(&index_sql)?;

            let mut buffer = Vec::with_capacity(self.batch_size * 2);
            let mut count = 0;

            for (id, name) in entries {
                buffer.push(duckdb::types::Value::Int(id as i32));
                buffer.push(duckdb::types::Value::Text(name.clone()));

                count += 1;

                if count % self.batch_size == 0 {
                    self.insert_enum_batch(&full_table_name, &buffer)?;
                    buffer.clear();
                }
            }

            if !buffer.is_empty() {
                self.insert_enum_batch(&full_table_name, &buffer)?;
            }
        }

        Ok(())
    }

    fn insert_enum_batch(&self, table_name: &str, buffer: &[duckdb::types::Value]) -> Result<()> {
        let num_rows = buffer.len() / 2;
        let placeholders = std::iter::repeat("(?, ?)").take(num_rows).collect::<Vec<_>>().join(",\n");
        let insert_sql = format!("INSERT INTO {} (Id, Name) VALUES\n{};", table_name, placeholders);
        self.connection.execute(&insert_sql, duckdb::params_from_iter(buffer.iter().cloned()))?;
        Ok(())
    }

    fn transfer_sqlite_to_duckdb_by_attach(&self, sqlite_path: &Path, schema_name: &str) -> Result<()> {
        let file_name = sqlite_path.file_stem().unwrap().to_string_lossy();
        let sqlite_path_str = sqlite_path.to_string_lossy().to_string();
        let connection = SqliteDb::new(sqlite_path)?;
        let table_names = connection.get_table_names()?;
        let mut query = format!("ATTACH '{}' (TYPE sqlite);\n", sqlite_path_str).to_string();

        for table_name in table_names {
            query += &format!("CREATE TABLE {}.{} AS SELECT * FROM {}.{};\n",
                schema_name, &table_name, file_name, &table_name);
        }
        
        query += &format!("DETACH {};", file_name);
        self.connection.execute_batch(&query)?;

        Ok(())
    }

    fn transfer_sqlite_to_duckdb_by_insert(&self, sqlite_path: &Path, schema_name: &str) -> Result<()> {
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
}

pub fn collect_db_file_entries(dir: &Path) -> Result<HashMap<String, DbFileEntry>> {
    let mut entries = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.extension().and_then(|s| s.to_str()) != Some("db") {
            continue;
        }

        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();

        entries.insert(
            file_stem.to_string(),
            DbFileEntry {
                file_path: file_path,
                file_name: file_name,
                strategy: TransformationStrategy::AttachSqlite,
            },
        );
    }

    Ok(entries)
}