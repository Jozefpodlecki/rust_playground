use std::{collections::HashMap, fs::{self, File}, io::Write, path::Path};
use anyhow::*;
use duckdb::{params, Connection as DuckConnection, Statement};
use log::info;

enum ColumnAction {
    Rename(String),
    Drop,
}

pub struct DuckDb(DuckConnection);

impl DuckDb {
    pub fn new(path: &Path) -> Result<Self> {
        let connection = DuckConnection::open(path)?;
        connection.execute("SET default_block_size = 32768;", [])?;

        Ok(Self(connection))
    }

    pub fn execute_script(&self, script_path: &Path) -> Result<()> {
         let sql = fs::read_to_string(script_path)
            .map_err(|e| anyhow!("Failed to read script file: {:?}", e))?;

        self.0.execute_batch(&sql);

        Ok(())
    }

    pub fn prepare_post_work_script(&self, output_path: &Path) -> Result<()> {

        let mut result = String::from("");

        info!("generate_drop_empty_columns");
        let comment_column_map = self.fetch_column_values("Comment")?;
        let script = &self.generate_drop_empty_columns_script("Comment", comment_column_map);
        self.0.execute_batch(script);
        result += script;

        info!("drop_empty_tables");
        let script = &self.generate_drop_empty_tables_script()?;
        self.0.execute_batch(script);
        result += script;
        
        info!("drop_unused_secondary_keys");
        let script = &self.generate_drop_unused_secondary_keys_script("SecondaryKey")?;
        self.0.execute_batch(script);
        result += script;

        info!("rename PrimaryKey");
        let script = &self.generate_column_script("PrimaryKey", ColumnAction::Rename("Id".to_string()))?;
        self.0.execute_batch(script);
        result += script;

        info!("rename SecondaryKey");
        let script = &self.generate_column_script("SecondaryKey", ColumnAction::Rename("SubId".to_string()))?;
        self.0.execute_batch(script);
        result += script;

        info!("drop Milestone");
        let script = &self.generate_column_script("Milestone", ColumnAction::Drop)?;
        self.0.execute_batch(script);
        result += script;

        info!("drop SourceRow");
        let script = &self.generate_column_script("SourceRow", ColumnAction::Drop)?;
        self.0.execute_batch(script);
        result += script;

        info!("integer_downgrade");
        let script = &self.generate_integer_downgrade_script()?;
        self.0.execute_batch(script);
        result += script;

        info!("primary_keys");
        let script = &self.generate_primary_keys_script()?;
        self.0.execute_batch(script);
        result += script;

        self.0.execute_batch("VACUUM;")?;

        let mut file = File::create(output_path)?;
        file.write_all(result.as_bytes())?;

        Ok(())
    }

    pub fn generate_integer_downgrade_script(&self) -> Result<String> {
        let sql = r#"
            SELECT table_schema, table_name, column_name
            FROM information_schema.columns
            WHERE data_type = 'BIGINT'
            AND table_schema != 'enums';
        "#;

        let mut stmt = self.0.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut script = String::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;
            let column: String = row.get(2)?;

            let range_sql = format!(
                r#"SELECT MIN("{col}"), MAX("{col}") FROM "{schema}"."{table}";"#,
                col = column,
                schema = schema,
                table = table
            );

            let mut range_stmt = self.0.prepare(&range_sql)?;
            let (min_val, max_val): (Option<i64>, Option<i64>) =
                range_stmt.query_row([], |r| std::result::Result::Ok((r.get(0)?, r.get(1)?)))?;

            let (Some(min), Some(max)) = (min_val, max_val) else {
                continue;
            };

            let target_type = if min == 0 && max == 1 {
                "BOOLEAN"
            } else if min >= i8::MIN as i64 && max <= i8::MAX as i64 {
                "TINYINT"
            } else if min >= i16::MIN as i64 && max <= i16::MAX as i64 {
                "SMALLINT"
            } else if min >= i32::MIN as i64 && max <= i32::MAX as i64 {
                "INTEGER"
            } else {
                continue;
            };

            script += &format!(
                "ALTER TABLE \"{}\".\"{}\" ALTER COLUMN \"{}\" SET DATA TYPE {};\n",
                schema, table, column, target_type
            );
        }

        Ok(script)
    }

    fn generate_primary_keys_script(&self) -> Result<String> {
        let sql = r#"
            SELECT
                table_schema,
                table_name,
                STRING_AGG(column_name, ',') AS columns
            FROM information_schema.columns
            WHERE column_name IN ('Id', 'SubId')
            GROUP BY table_schema, table_name
            ORDER BY table_schema, table_name;
        "#;

        let mut stmt = self.0.prepare(sql)?;
        let mut rows = stmt.query([])?;
        let mut script = String::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;
            let column: String = row.get(2)?;

            script += &format!(
                "ALTER TABLE \"{}\".\"{}\" ADD PRIMARY KEY (\"{}\");\n",
                schema, table, column
            );
        }

        Ok(script)
    }

    pub fn fetch_column_values(&self, column_name: &str) -> Result<HashMap<String, Vec<String>>> {
        let query = r#"
            SELECT 
                table_schema,
                table_name
            FROM information_schema.columns
            WHERE column_name = ?
        "#;

        let mut stmt = self.0.prepare(query)?;
        let mut rows = stmt.query([column_name])?;

        let mut results: HashMap<String, Vec<String>> = HashMap::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;

            let select_sql = format!(
                r#"SELECT "{}" FROM "{}"."{}";"#,
                column_name, schema, table
            );

            let mut select_stmt = self.0.prepare(&select_sql)?;
            let mut value_rows = select_stmt.query([])?;

            let mut values = Vec::new();

            while let Some(value_row) = value_rows.next()? {
                let value: Option<String> = value_row.get(0)?;
                if let Some(val) = value {
                    if !val.trim().is_empty() {
                        values.push(val);
                    }
                }
            }

            let key = format!("{}.{}", schema,table);
            results.insert(key, values);
        }

        Ok(results)
    }

    pub fn generate_drop_empty_columns_script(
        &self,
        column_name: &str,
        values_by_table: HashMap<String, Vec<String>>,
    ) -> String {
        let mut script = String::new();

        for (table_name, values) in values_by_table {
            if values.is_empty() {
                let alter_stmt = format!(
                    r#"ALTER TABLE {} DROP "{}";"#,
                    table_name, column_name
                );
                script.push_str(&alter_stmt);
                script.push('\n');
            }
        }

        script
    }

    fn generate_column_script(&self, column_name: &str, action: ColumnAction) -> Result<String> {
        let query = r#"
            SELECT 
                table_schema,
                table_name
            FROM information_schema.columns
            WHERE column_name = ?
        "#;

        let mut stmt = self.0.prepare(query)?;
        let mut rows = stmt.query([column_name])?;

        let mut script = String::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;

            let alter_stmt = match &action {
                ColumnAction::Rename(to_name) => format!(
                    r#"ALTER TABLE "{}"."{}" RENAME "{}" TO "{}";"#,
                    schema, table, column_name, to_name
                ),
                ColumnAction::Drop => format!(
                    r#"ALTER TABLE "{}"."{}" DROP COLUMN "{}";"#,
                    schema, table, column_name
                ),
            };

            script.push_str(&alter_stmt);
            script.push('\n');
        }

        Ok(script)
    }

    pub fn generate_drop_empty_tables_script(&self) -> Result<String> {
        let query = r#"
            SELECT 
                table_schema,
                table_name
            FROM information_schema.tables
            WHERE table_type = 'BASE TABLE'
        "#;

        let mut stmt = self.0.prepare(query)?;
        let mut rows = stmt.query([])?;

        let mut script = String::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;

            let count_query = format!(
                r#"SELECT COUNT(*) FROM "{}"."{}""#,
                schema, table
            );

            let count: i64 = self.0.query_row(&count_query, [], |row| row.get(0))?;

            if count == 0 {
                let drop_stmt = format!(r#"DROP TABLE "{}"."{}";"#, schema, table);
                script.push_str(&drop_stmt);
                script.push('\n');
            }
        }

        Ok(script)
    }

    pub fn generate_drop_unused_secondary_keys_script(&self, column_name: &str) -> Result<String> {
        let query = format!(
            r#"
            SELECT table_schema, table_name
            FROM information_schema.columns
            WHERE column_name = '{}';
            "#,
            column_name
        );

        let mut statement = self.0.prepare(&query)?;
        let mut rows = statement.query([])?;
        let mut script = String::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;

            let query = format!(
                "SELECT COUNT(DISTINCT \"{}\") FROM \"{}\".\"{}\";",
                column_name, schema, table
            );

            let mut check_stmt = self.0.prepare(&query)?;
            let distinct_count: i64 = check_stmt.query_row([], |r| r.get(0))?;

            if distinct_count == 1 {
                script += &format!("ALTER TABLE \"{}\".\"{}\" DROP {};\n", schema, table, column_name);
            }
        }

        Ok(script)
    }
}