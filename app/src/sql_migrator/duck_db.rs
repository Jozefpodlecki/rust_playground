use std::{collections::HashMap, path::Path};
use anyhow::*;
use duckdb::{CachedStatement, Connection as DuckConnection, Params};

use crate::sql_migrator::{types::ColumnAction, utils::{to_camel_case, to_snake_case}};

pub struct DuckDb(DuckConnection);

impl DuckDb {
    pub fn new(path: &Path) -> Result<Self> {
        let connection = DuckConnection::open(path)?;
        connection.execute("SET default_block_size = 32768;", [])?;

        Ok(Self(connection))
    }

    pub fn execute_batch(&self, sql: &str) -> Result<()> {
        self.0.execute_batch(sql);
        Ok(())
    }

    pub fn execute<P: Params>(&self, sql: &str, params: P) -> Result<()> {
        self.0.execute(sql, params)?;
        Ok(())
    }

    pub fn process_skill(&self) -> Result<String> {
        let query = r"SELECT * FROM data.Skill";

        let script = "".into();

        let mut statement = self.0.prepare(query)?;
        let column_names: Vec<(usize, String)> = statement
            .column_names()
            .iter()
            .filter(|&pr| pr != "Id" && "pr" != "SubId" )
            .enumerate()
            .map(|(i, name)| (i, name.to_string()))
            .collect();
        let mut rows = statement.query([])?;
        let records: Vec<HashMap<String, serde_json::Value>> = vec![];

        while let Some(row) = rows.next()? {
            let mut map: HashMap<String, serde_json::Value> = HashMap::new();

            for (it, name) in column_names.clone() {
                let value = row.get_ref(it)?;
                let json_value = match value {
                    duckdb::types::ValueRef::Null => serde_json::Value::Null,
                    duckdb::types::ValueRef::Boolean(value) => {
                        if !value {
                            serde_json::Value::Null
                        }
                        else {
                            serde_json::Value::Bool(value)
                        }
                    }
                    duckdb::types::ValueRef::TinyInt(value) => {
                        if value == 0 {
                            serde_json::Value::Null
                        }
                        else {
                            let value = value.into();
                            serde_json::Value::Number(value)
                        }
                    },
                    duckdb::types::ValueRef::SmallInt(value) => {
                        if value == 0 {
                            serde_json::Value::Null
                        }
                        else {
                            let value = value.into();
                            serde_json::Value::Number(value)
                        }
                    },
                    duckdb::types::ValueRef::Int(value) => {
                        if value == 0 {
                            serde_json::Value::Null
                        }
                        else {
                            let value = value.into();
                            serde_json::Value::Number(value)
                        }
                    },
                    duckdb::types::ValueRef::BigInt(value) => {
                        if value == 0 {
                            serde_json::Value::Null
                        }
                        else {
                            let value = value.into();
                            serde_json::Value::Number(value)
                        }
                    },
                    duckdb::types::ValueRef::Text(bytes) => {

                        let value = String::from_utf8_lossy(bytes).to_string();

                        if value.is_empty() {
                            serde_json::Value::Null
                        }
                        else {
                            serde_json::Value::String(value)
                        }
                    }
                    _ => todo!()
                };

                if !json_value.is_null() {
                    map.insert(name, json_value);
                }
            }
            
        }

        Ok(script)
    }

    pub fn prepare(&self, sql: &str) -> Result<CachedStatement<'_>> {
        let statement = self.0.prepare_cached(sql)?;
        Ok(statement)
    }

    pub fn tables_exists(&self, schema: &str) -> Result<bool> {
        let query = r#"
            SELECT COUNT(*) 
            FROM information_schema.tables 
            WHERE table_schema = ?;
        "#;

        let count: i64 = self.0.query_row(query, &[schema], |row| row.get(0))?;

        Ok(count > 0)
    }

    pub fn generate_global_search_view(&self) -> Result<String> {
        let query = r#"
            SELECT table_schema, table_name
            FROM information_schema.columns
            WHERE column_name = 'Id'
            AND data_type = 'bigint'
            AND table_schema != 'enums';
            "#;

        let mut statement = self.0.prepare(query)?;
        let mut rows = statement.query([])?;

        let mut selects = Vec::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;
            let column: String = row.get(2)?;

            let select = format!(
                "SELECT Id, '{}' AS table_name FROM {}.{}",
                table, schema, table
            );
            selects.push(select);
        }

        let union_query = selects.join(" UNION ALL ");

        let create_view = format!("CREATE OR REPLACE VIEW data.IdLookup AS {}", union_query);

        Ok(create_view)
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

    pub fn generate_primary_keys_script(&self) -> Result<String> {
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

    pub fn get_column_values(&self, column_name: &str) -> Result<HashMap<String, Vec<String>>> {
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

            let mut statement = self.0.prepare(&select_sql)?;
            let mut value_rows = statement.query([])?;

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

    pub fn generate_rust_struct(&self, table_schema: &str, table_name: &str) -> Result<String> {
        let query = r#"
            SELECT column_name, data_type
            FROM information_schema.columns
            WHERE table_schema = ? AND table_name = ?
            ORDER BY ordinal_position
        "#;

        let mut statement = self.0.prepare(query)?;
        let mut rows = statement.query([table_schema, table_name])?;

        let mut fields = Vec::new();
        let mut assignments = Vec::new();

        let mut index = 0;
        while let Some(row) = rows.next()? {
            let column_name: String = row.get(0)?;
            let data_type: String = row.get(1)?;

            let rust_type = match data_type.as_str() {
                "TINYINT" => "u8",
                "SMALLINT" => "i16",
                "INTEGER" => "i32",
                "BIGINT" => "i64",
                "BOOLEAN" => "bool",
                "VARCHAR" => "String",
                _ => return Err(anyhow!("Unsupported data type: {}", data_type)),
            };

            let field_name = to_snake_case(&column_name);
            fields.push(format!("    pub {}: {},", field_name, rust_type));
            assignments.push(format!("            {}: row.get({})?,", field_name, index));

            index += 1;
        }

        let struct_name = to_camel_case(table_name);

        let struct_def = format!(
        r#"pub struct {name} {{
        {fields}
        }}

        impl {name} {{
            pub fn from(row: duckdb::Row) -> anyhow::Result<Self> {{
                Ok(Self {{
        {assignments}
                }})
            }}
        }}"#,
            name = struct_name,
            fields = fields.join("\n"),
            assignments = assignments.join("\n")
        );

        Ok(struct_def)
    }

    pub fn generate_column_script(&self, column_name: &str, action: ColumnAction) -> Result<String> {
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

    pub fn generate_update_localization_script(&self) -> Result<String> {
        let query = r#"
            SELECT 
                table_schema,
                table_name,
                column_name
            FROM information_schema.columns
            WHERE column_name IN ('Name', 'Desc', 'DescPvp')
        "#;

        let mut statement = self.0.prepare(query)?;
        let mut rows = statement.query([])?;
        let mut table_columns: HashMap<String, Vec<String>> = HashMap::new();

        while let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            let table: String = row.get(1)?;
            let column: String = row.get(2)?;
            let key = format!("{}.{}", schema, table);

            table_columns
                .entry(key)
                .or_default()
                .push(column);
        }

        let mut script = String::new();

        for (table_name, columns) in table_columns {
            
            for column_name in columns.iter() {
                let stmt = format!(
                    r#"UPDATE {} t
                    SET "{}" = gme.Message
                    FROM data.GameMsg_English gme
                    WHERE gme.Key = t.{};"#,
                    table_name, column_name, column_name
                );
                script.push_str(&stmt);
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