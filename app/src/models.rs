/*
|ordinal_position|table_schema|table_name|column_name|data_type|
|----------------|------------|----------|-----------|---------|
|1|data|Ability|Id|INTEGER|
|2|data|Ability|SubId|TINYINT|
|3|data|Ability|ClassifyType|BOOLEAN|
|4|data|Ability|ClassifyIndex|SMALLINT|
|5|data|Ability|ClassifyType2|TINYINT|
|6|data|Ability|ClassifyIndex2|TINYINT|
|7|data|Ability|Name|VARCHAR|
|8|data|Ability|InvokeName|VARCHAR|
|9|data|Ability|Comment1|VARCHAR|
|10|data|Ability|Desc|VARCHAR|
|11|data|Ability|Comment2|VARCHAR|
|12|data|Ability|DescPvp|VARCHAR|
|13|data|Ability|Penalty|BOOLEAN|
|14|data|Ability|ReqPoint|TINYINT|
|15|data|Ability|AbilityType|TINYINT|
|16|data|Ability|ExclusiveClass|SMALLINT|
|17|data|Ability|Icon|VARCHAR|
|18|data|Ability|IconIndex|SMALLINT|
|19|data|Ability|Auction|BOOLEAN|
|20|data|Ability|DummyStatusEffectId|TINYINT|
|21|data|Ability|OptionType00|TINYINT|
|22|data|Ability|OptionKeyStat00|TINYINT|
|23|data|Ability|OptionKeyIndex00|INTEGER|
|24|data|Ability|OptionValue00|SMALLINT|
|25|data|Ability|OptionType01|TINYINT|
|26|data|Ability|OptionKeyStat01|SMALLINT|
|27|data|Ability|OptionKeyIndex01|INTEGER|
|28|data|Ability|OptionValue01|SMALLINT|
|29|data|Ability|OptionType02|TINYINT|
|30|data|Ability|OptionKeyStat02|SMALLINT|
|31|data|Ability|OptionKeyIndex02|INTEGER|
|32|data|Ability|OptionValue02|SMALLINT|
|33|data|Ability|IsEngraveAbility|BOOLEAN|
|34|data|Ability|Season|TINYINT|
*/

use anyhow::*;
use duckdb::Row;

pub struct Ability {
    pub id: i32,
}

impl Ability {
    pub fn from(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.get(0)?
        })
    }
}