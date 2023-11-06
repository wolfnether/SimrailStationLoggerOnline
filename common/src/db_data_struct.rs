use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct Log {
    pub server: String,
    pub station: String,
    pub player: String,
    pub date: DateTime<Utc>,
}
