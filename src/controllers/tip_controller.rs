use anyhow::Result;
use std::time::Instant;
use uuid::Uuid;

use crate::db::connection::AppState;
use crate::db::query_logger::QueryLogger;
use crate::models::tip::{RecordTipRequest, Tip};
use crate::cache::{redis_client, keys};
use crate::controllers::creator_controller;
use crate::email::EmailMessage;
use tera::Context;

pub async fn record_tip(state: &AppState, req: RecordTipRequest) -> Result<Tip> {
    let query = r#"
        INSERT INTO tips (id, creator_username, amount, transaction_hash, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id, creator_username, amount, transaction_hash, created_at
        "#;
    
    let start = Instant::now();
    let tip = sqlx::query_as::<_, Tip>(query)
    .bind(Uuid::new_v4())
    .bind(&req.username)
    .bind(&req.amount)
    .bind(&req.transaction_hash)
    .fetch_one(&state.db)
    .await?;
    let duration = start.elapsed();

    QueryLogger::log_query(query, duration);
    state.performance.track_query(query, duration);

    // Invalidate the tip list cache for this creator since it's now stale.
    if let Some(conn) = state.redis.as_ref() {
        let mut conn = conn.clone();
        let tips_key = keys::creator_tips(&tip.creator_username);
        redis_client::del(&mut conn, &[tips_key.as_str()]).await;
    }

    // Send email notification if creator has an email address configured.
    let tip_clone = tip.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Ok(Some(creator)) = creator_controller::get_creator_by_username(&state_clone, &tip_clone.creator_username).await {
            if let Some(email_addr) = creator.email {
                let mut context = Context::new();
                context.insert("username", &creator.username);
                context.insert("amount", &tip_clone.amount.to_string());
                context.insert("transaction_hash", &tip_clone.transaction_hash);

                let email_msg = EmailMessage {
                    to: email_addr,
                    subject: format!("🚀 You've received a new tip of {} XLM!", tip_clone.amount),
                    template_name: "tip_received.html".into(),
                    context,
                };

                if let Err(e) = state_clone.email.send(email_msg).await {
                    tracing::error!("Failed to queue tip notification email: {}", e);
                }
            }
        }
    });

    Ok(tip)
}

pub async fn get_tips_for_creator(state: &AppState, username: &str) -> Result<Vec<Tip>> {
    let query = r#"
        SELECT id, creator_username, amount, transaction_hash, created_at
        FROM tips
        WHERE creator_username = $1
        ORDER BY created_at DESC
        "#;

    let start = Instant::now();
    let tips = sqlx::query_as::<_, Tip>(query)
    .bind(username)
    .fetch_all(&state.db)
    .await?;
    let duration = start.elapsed();

    QueryLogger::log_query(query, duration);
    state.performance.track_query(query, duration);

    // Populate cache.
    if let Some(conn) = state.redis.as_ref() {
        let mut conn = conn.clone();
        let cache_key = keys::creator_tips(username);
        redis_client::set(&mut conn, &cache_key, &tips, redis_client::TTL_TIPS).await;
    }

    Ok(tips)
}
