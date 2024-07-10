use std::collections::HashMap;

use super::Challenge;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ctf {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub server_time: DateTime<Utc>,
    pub freeze_start: Option<DateTime<Utc>>,
    pub freeze_end: Option<DateTime<Utc>>,
    pub teams: bool,
    pub challenges_by_category: HashMap<String, Vec<Challenge>>,
}
