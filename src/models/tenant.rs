use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub max_creators: i32,
    pub max_tips_per_day: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be 2–100 characters"))]
    pub name: String,
    #[validate(length(min = 2, max = 50, message = "Slug must be 2–50 characters"))]
    #[validate(regex(
        path = "SLUG_REGEX",
        message = "Slug may only contain lowercase letters, numbers, and hyphens"
    ))]
    pub slug: String,
    pub max_creators: Option<i32>,
    pub max_tips_per_day: Option<i32>,
}

lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTenantRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
    pub max_creators: Option<i32>,
    pub max_tips_per_day: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub max_creators: i32,
    pub max_tips_per_day: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Tenant> for TenantResponse {
    fn from(t: Tenant) -> Self {
        Self {
            id: t.id,
            name: t.name,
            slug: t.slug,
            max_creators: t.max_creators,
            max_tips_per_day: t.max_tips_per_day,
            is_active: t.is_active,
            created_at: t.created_at,
        }
    }
}
