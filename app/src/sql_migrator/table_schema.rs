
use anyhow::*;
use crate::sql_migrator::utils::*;

#[derive(Debug)]
pub struct TableColumn {
    pub order: usize,
    pub name: String,
    pub col_type: String,
    pub mapped_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<TableColumn>,
}

impl TableSchema {
    pub fn from_sqlite(connection: &rusqlite::Connection, table_name: &str) -> anyhow::Result<Self> {
        let pragma = format!("PRAGMA table_info({})", table_name);
        let mut statement = connection.prepare(&pragma)?;
        let rows = statement.query_map([], |row| {
            let order: usize = row.get(0)?;
            let original_type: String = row.get(2)?;
            let name: String = row.get(1)?;
            let mapped_type = map_sqlite_type_to_duckdb(table_name, &name, &original_type).to_string();
            rusqlite::Result::Ok(TableColumn {
                order,
                name,
                col_type: original_type.clone(),
                mapped_type,
                nullable: row.get::<_, i32>(3)? == 0,
                is_primary_key: row.get::<_, i32>(5)? > 0,
            })
        })?;

        let mut columns = Vec::new();
        for col in rows {
            columns.push(col?);
        }

        Ok(Self {
            name: table_name.to_string(),
            columns,
        })
    }

    pub fn to_create_table_sql(&self, schema: &str) -> String {
        let defs: Vec<String> = self.columns.iter().map(|col| {
            let col_name = rename_column(&col.name);
            format!("{} {}", col_name, col.mapped_type)
        }).collect();

        let pk_cols: Vec<String> = self.columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| rename_column(&c.name).to_string())
            .collect();

        if !pk_cols.is_empty() {
            // defs.push(format!("PRIMARY KEY ({})", pk_cols.join(", ")));
        }

        format!(
            "CREATE TABLE IF NOT EXISTS {}.{} (\n    {}\n);",
            schema,
            self.name,
            defs.join(",\n    ")
        )
    }

    pub fn column_names(&self) -> Vec<String> {
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}