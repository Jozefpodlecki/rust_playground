use chrono::Duration;
use duckdb::{types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef}, ToSql};

#[derive(Debug)]
pub struct CustomDuration {
    raw: u64
}

impl From<u64> for CustomDuration {
    fn from(seconds: u64) -> Self {
        Self { raw: seconds }
    }
}

impl FromSql for CustomDuration {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        if let ValueRef::Interval { months: _, days: _, nanos } = value {
            let duration = Duration::nanoseconds(nanos);

            return Ok(CustomDuration::new(duration.num_seconds() as u64))
        }
        
        Err(FromSqlError::InvalidType)
    }
}

impl ToSql for CustomDuration {
    fn to_sql(&self) -> duckdb::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_mm_ss()))
    }
}

impl CustomDuration {
    pub fn new (seconds: u64) -> Self {
        Self { raw: seconds }
    }

    fn to_mm_ss(&self) -> String {
        let minutes = self.raw / 60;
        let seconds = self.raw % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {

    use duckdb::ToSql;

    use super::CustomDuration;

    #[test]
    fn test_hp_session() {

        let duration: CustomDuration = 61.into();
        let sql = duration.to_sql().unwrap();
    }
}