pub fn rename_column(name: &str) -> String {
    match name {
        "Desc" => "Description".into(),
        _ => format!("\"{name}\"")
    }
}

pub fn map_sqlite_type_to_duckdb<'a>(table_name: &'a str, column_name: &'a str, sqlite_type: &'a str) -> &'a str {
    match (table_name, column_name, sqlite_type) {
        
        (_, "MinAmount", "INTEGER") => "BIGINT",
        (_, "MaxAmount", "INTEGER") => "BIGINT",
        (_, _, "INTEGER") => "INT",
        (_, _, "TEXT") => "VARCHAR(100)",
        _ => "VARCHAR(100)"
    }
}

pub fn value_ref_to_duckdb_param(column_type: &str, value: rusqlite::types::ValueRef) -> duckdb::types::Value {
    match value {
        rusqlite::types::ValueRef::Null => duckdb::types::Value::Null,
        rusqlite::types::ValueRef::Integer(i) => duckdb::types::Value::BigInt(32),
        rusqlite::types::ValueRef::Real(f) => duckdb::types::Value::Double(f),
        rusqlite::types::ValueRef::Text(t) => {

            if column_type == "INTEGER" {
                return duckdb::types::Value::BigInt(32)
            }

            let string_value = String::from_utf8_lossy(t).into_owned();
            duckdb::types::Value::Text(string_value)
        }
        rusqlite::types::ValueRef::Blob(blob) => duckdb::types::Value::Blob(blob.to_owned()),
    }
}