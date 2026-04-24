use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TipDailyStat {
    pub creator_username: String,
    pub stat_date: NaiveDate,
    pub tip_count: i64,
    pub total_amount: String,
    pub avg_amount: String,
    pub max_amount: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TipSummary {
    pub creator_username: String,
    pub total_tips: i64,
    pub total_amount: String,
    pub avg_amount: String,
    pub max_amount: String,
}

#[derive(Debug, Deserialize)]
pub struct StatsQuery {
    /// Number of days to look back (default 30, max 365)
    #[serde(default = "StatsQuery::default_days")]
    pub days: i64,
}

impl StatsQuery {
    fn default_days() -> i64 {
        30
    }
    pub fn clamped_days(&self) -> i64 {
        self.days.clamp(1, 365)
    }
}
