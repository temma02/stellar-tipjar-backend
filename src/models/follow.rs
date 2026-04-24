use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CreatorFollow {
    pub follower_username: String,
    pub followed_username: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FollowCount {
    pub creator_username: String,
    pub follower_count: i64,
    pub following_count: i64,
}
