use sqlx::PgPool;

use crate::errors::AppResult;
use crate::models::stats::{StatsQuery, TipDailyStat, TipSummary};

pub async fn get_creator_summary(pool: &PgPool, username: &str) -> AppResult<TipSummary> {
    let summary = sqlx::query_as::<_, TipSummary>(
        r#"
        SELECT
            $1::TEXT AS creator_username,
            COUNT(*)::BIGINT AS total_tips,
            COALESCE(SUM(amount::NUMERIC), 0)::TEXT AS total_amount,
            COALESCE(AVG(amount::NUMERIC), 0)::TEXT AS avg_amount,
            COALESCE(MAX(amount::NUMERIC), 0)::TEXT AS max_amount
        FROM tips
        WHERE creator_username = $1
        "#,
    )
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(summary)
}

pub async fn get_daily_stats(
    pool: &PgPool,
    username: &str,
    query: &StatsQuery,
) -> AppResult<Vec<TipDailyStat>> {
    let days = query.clamped_days();
    let stats = sqlx::query_as::<_, TipDailyStat>(
        r#"
        SELECT
            creator_username,
            DATE(created_at) AS stat_date,
            COUNT(*)::BIGINT AS tip_count,
            COALESCE(SUM(amount::NUMERIC), 0)::TEXT AS total_amount,
            COALESCE(AVG(amount::NUMERIC), 0)::TEXT AS avg_amount,
            COALESCE(MAX(amount::NUMERIC), 0)::TEXT AS max_amount
        FROM tips
        WHERE creator_username = $1
          AND created_at >= NOW() - ($2 || ' days')::INTERVAL
        GROUP BY creator_username, DATE(created_at)
        ORDER BY stat_date DESC
        "#,
    )
    .bind(username)
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(stats)
}

/// Upsert aggregated daily stats into the tip_daily_stats table.
pub async fn aggregate_daily_stats(pool: &PgPool, username: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO tip_daily_stats (creator_username, stat_date, tip_count, total_amount, avg_amount, max_amount)
        SELECT
            creator_username,
            DATE(created_at),
            COUNT(*),
            COALESCE(SUM(amount::NUMERIC), 0),
            COALESCE(AVG(amount::NUMERIC), 0),
            COALESCE(MAX(amount::NUMERIC), 0)
        FROM tips
        WHERE creator_username = $1
        GROUP BY creator_username, DATE(created_at)
        ON CONFLICT (creator_username, stat_date) DO UPDATE SET
            tip_count    = EXCLUDED.tip_count,
            total_amount = EXCLUDED.total_amount,
            avg_amount   = EXCLUDED.avg_amount,
            max_amount   = EXCLUDED.max_amount
        "#,
    )
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}
