use std::mem::transmute;

use duckdb::{types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef}, ToSql};

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum NpcType {
    Boss = 0
}

impl ToSql for NpcType {
    fn to_sql(&self) -> duckdb::Result<ToSqlOutput<'_>> {
        let value = *self as u8;

        Ok(ToSqlOutput::from(value))
    }
}

impl FromSql for NpcType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        if let ValueRef::UTinyInt(value) = value {
            let value: NpcType = unsafe { transmute(value) };
            return Ok(value);
        }
        
        Err(FromSqlError::InvalidType)
    }
}