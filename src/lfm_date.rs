use std::{collections::HashMap, ops::Deref};

use chrono::{DateTime, LocalResult, TimeZone, Utc};
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct LfmDate(DateTime<Utc>);

impl<'de> Deserialize<'de> for LfmDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_data: HashMap<String, String> = Deserialize::deserialize(deserializer)?;

        let uts = raw_data
            .get("uts")
            .ok_or_else(|| D::Error::missing_field("uts"))?
            .parse::<i64>()
            .map_err(|_| D::Error::custom("Failed to parse uts as i64"))?;

        let local_result = Utc.timestamp_opt(uts, 0);
        if let LocalResult::Single(date_time) = local_result {
            Ok(LfmDate(date_time))
        } else {
            Err(D::Error::custom("Failed to parse uts as i64"))
        }
    }
}

impl Deref for LfmDate {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_deserializes_correctly() {
        let json_value = json!({
          "uts": "1676284092",
          "#text": "13 Feb 2023, 10:28"
        });

        let lfm_date: LfmDate = serde_json::from_value(json_value).unwrap();
        let expected = "2023-02-13 10:28:12 UTC";
        assert_eq!(lfm_date.to_string(), expected);
    }
}
