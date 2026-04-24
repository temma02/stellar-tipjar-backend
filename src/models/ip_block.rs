use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IpBlock {
    pub id: Uuid,
    pub ip_address: String,
    pub reason: Option<String>,
    pub blocked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CountryBlock {
    pub country_code: String,
    pub reason: Option<String>,
    pub blocked_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct BlockIpRequest {
    pub ip_address: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct BlockCountryRequest {
    pub country_code: String,
    pub reason: Option<String>,
}
