//! Persistent review queue for flagged content.
//!
//! Flagged items are stored in the `moderation_queue` table and surfaced to
//! administrators through the admin API. Reviewers can approve or reject each
//! item, recording who took the action and when.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::{ContentType, ModerationResult};

/// A row from the `moderation_queue` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ModerationQueueItem {
    pub id: Uuid,
    pub content_type: String,
    pub content_id: Option<Uuid>,
    pub content_text: String,
    /// JSON array of [`Violation`] objects.
    pub flags: serde_json::Value,
    /// `pending` | `approved` | `rejected`
    pub status: String,
    pub ai_score: Option<f64>,
    pub ai_reasoning: Option<String>,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Summary counts returned by the stats endpoint.
#[derive(Debug, Serialize)]
pub struct ModerationStats {
    pub pending: i64,
    pub approved: i64,
    pub rejected: i64,
    pub total: i64,
}

pub struct ReviewQueue {
    db: PgPool,
}

impl ReviewQueue {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Persist a flagged item to the review queue and return the new row ID.
    pub async fn enqueue(
        &self,
        content: &str,
        content_type: &ContentType,
        content_id: Option<Uuid>,
        result: &ModerationResult,
    ) -> anyhow::Result<Uuid> {
        let flags = serde_json::to_value(&result.violations)
            .unwrap_or(serde_json::Value::Array(vec![]));

        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO moderation_queue
                (id, content_type, content_id, content_text, flags, status, ai_score, ai_reasoning, created_at)
            VALUES
                ($1, $2, $3, $4, $5, 'pending', $6, $7, NOW())
            RETURNING id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(content_type.as_str())
        .bind(content_id)
        .bind(content)
        .bind(flags)
        .bind(result.ai_score.map(|s| s as f64))
        .bind(&result.ai_reasoning)
        .fetch_one(&self.db)
        .await?;

        Ok(id)
    }

    /// List items by status. Pass `None` to retrieve all statuses.
    pub async fn list(
        &self,
        status: Option<&str>,
        limit: i64,
    ) -> anyhow::Result<Vec<ModerationQueueItem>> {
        let items = match status {
            Some(s) => {
                sqlx::query_as::<_, ModerationQueueItem>(
                    r#"
                    SELECT id, content_type, content_id, content_text, flags, status,
                           ai_score, ai_reasoning, reviewed_by, reviewed_at, created_at
                    FROM moderation_queue
                    WHERE status = $1
                    ORDER BY created_at DESC
                    LIMIT $2
                    "#,
                )
                .bind(s)
                .bind(limit)
                .fetch_all(&self.db)
                .await?
            }
            None => {
                sqlx::query_as::<_, ModerationQueueItem>(
                    r#"
                    SELECT id, content_type, content_id, content_text, flags, status,
                           ai_score, ai_reasoning, reviewed_by, reviewed_at, created_at
                    FROM moderation_queue
                    ORDER BY created_at DESC
                    LIMIT $1
                    "#,
                )
                .bind(limit)
                .fetch_all(&self.db)
                .await?
            }
        };
        Ok(items)
    }

    /// Approve a queued item. Returns `false` when the ID does not exist.
    pub async fn approve(&self, id: Uuid, reviewed_by: &str) -> anyhow::Result<bool> {
        let rows = sqlx::query(
            r#"
            UPDATE moderation_queue
            SET status = 'approved', reviewed_by = $1, reviewed_at = NOW()
            WHERE id = $2 AND status = 'pending'
            "#,
        )
        .bind(reviewed_by)
        .bind(id)
        .execute(&self.db)
        .await?
        .rows_affected();

        Ok(rows > 0)
    }

    /// Reject a queued item. Returns `false` when the ID does not exist.
    pub async fn reject(&self, id: Uuid, reviewed_by: &str) -> anyhow::Result<bool> {
        let rows = sqlx::query(
            r#"
            UPDATE moderation_queue
            SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW()
            WHERE id = $2 AND status = 'pending'
            "#,
        )
        .bind(reviewed_by)
        .bind(id)
        .execute(&self.db)
        .await?
        .rows_affected();

        Ok(rows > 0)
    }

    /// Returns counts of items in each status bucket.
    pub async fn stats(&self) -> anyhow::Result<ModerationStats> {
        let pending: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM moderation_queue WHERE status = 'pending'")
                .fetch_one(&self.db)
                .await?;

        let approved: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM moderation_queue WHERE status = 'approved'")
                .fetch_one(&self.db)
                .await?;

        let rejected: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM moderation_queue WHERE status = 'rejected'")
                .fetch_one(&self.db)
                .await?;

        Ok(ModerationStats {
            pending,
            approved,
            rejected,
            total: pending + approved + rejected,
        })
    }
}
