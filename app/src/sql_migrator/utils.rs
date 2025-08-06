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
        rusqlite::types::ValueRef::Integer(i) => duckdb::types::Value::BigInt(i),
        rusqlite::types::ValueRef::Real(f) => duckdb::types::Value::Double(f),
        rusqlite::types::ValueRef::Text(t) => {

            if column_type == "INTEGER" {
                return duckdb::types::Value::BigInt(0)
            }

            let string_value = String::from_utf8_lossy(t).into_owned();
            duckdb::types::Value::Text(string_value)
        }
        rusqlite::types::ValueRef::Blob(blob) => duckdb::types::Value::Blob(blob.to_owned()),
    }
}

pub fn get_duckdb_int_type_for_enum_keys(max_id: u32) -> &'static str {

    if max_id <= i8::MAX as u32 {
        "TINYINT"
    } else if max_id <= i16::MAX as u32 {
        "SMALLINT"
    } else if max_id <= i32::MAX as u32 {
        "INT"
    } else {
        "BIGINT"
    }
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i != 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in s.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(ch);
        }
    }
    result
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let bytes_f64 = bytes as f64;

    if bytes_f64 >= TB {
        format!("{:.2} TB", bytes_f64 / TB)
    } else if bytes_f64 >= GB {
        format!("{:.2} GB", bytes_f64 / GB)
    } else if bytes_f64 >= MB {
        format!("{:.2} MB", bytes_f64 / MB)
    } else if bytes_f64 >= KB {
        format!("{:.2} KB", bytes_f64 / KB)
    } else {
        format!("{} B", bytes)
    }
}