use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(FromRow))]
pub struct Log {
    pub server: String,
    pub station: String,
    pub player: String,
    pub date: DateTime<Utc>,
}
