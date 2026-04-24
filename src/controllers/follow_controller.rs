use sqlx::PgPool;

use crate::errors::AppResult;
use crate::models::follow::{CreatorFollow, FollowCount};

pub async fn follow(pool: &PgPool, follower: &str, followed: &str) -> AppResult<CreatorFollow> {
    let follow = sqlx::query_as::<_, CreatorFollow>(
        r#"
        INSERT INTO creator_follows (follower_username, followed_username)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        RETURNING follower_username, followed_username, created_at
        "#,
    )
    .bind(follower)
    .bind(followed)
    .fetch_one(pool)
    .await?;
    Ok(follow)
}

pub async fn unfollow(pool: &PgPool, follower: &str, followed: &str) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM creator_follows WHERE follower_username = $1 AND followed_username = $2",
    )
    .bind(follower)
    .bind(followed)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_followers(pool: &PgPool, username: &str) -> AppResult<Vec<CreatorFollow>> {
    let follows = sqlx::query_as::<_, CreatorFollow>(
        "SELECT follower_username, followed_username, created_at FROM creator_follows WHERE followed_username = $1 ORDER BY created_at DESC",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(follows)
}

pub async fn get_following(pool: &PgPool, username: &str) -> AppResult<Vec<CreatorFollow>> {
    let follows = sqlx::query_as::<_, CreatorFollow>(
        "SELECT follower_username, followed_username, created_at FROM creator_follows WHERE follower_username = $1 ORDER BY created_at DESC",
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    Ok(follows)
}

pub async fn get_follow_counts(pool: &PgPool, username: &str) -> AppResult<FollowCount> {
    let counts = sqlx::query_as::<_, FollowCount>(
        r#"
        SELECT
            $1::TEXT AS creator_username,
            (SELECT COUNT(*) FROM creator_follows WHERE followed_username = $1)::BIGINT AS follower_count,
            (SELECT COUNT(*) FROM creator_follows WHERE follower_username = $1)::BIGINT AS following_count
        "#,
    )
    .bind(username)
    .fetch_one(pool)
    .await?;
    Ok(counts)
}

pub async fn get_feed(pool: &PgPool, username: &str, limit: i64) -> AppResult<Vec<String>> {
    // Returns recently tipped creators from people the user follows
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT t.creator_username
        FROM tips t
        JOIN creator_follows cf ON cf.followed_username = t.creator_username
        WHERE cf.follower_username = $1
        ORDER BY t.creator_username
        LIMIT $2
        "#,
    )
    .bind(username)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
