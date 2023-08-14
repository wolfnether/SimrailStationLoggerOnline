use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "backend")]
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(FromRow))]
struct Log {
    server: String,
    station: String,
    player: String,
    date: DateTime<Utc>,
}
