use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::controllers::tenant_controller;
use crate::db::connection::AppState;
use crate::errors::AppError;
use crate::models::tenant::{CreateTenantRequest, UpdateTenantRequest};

#[derive(Debug, Deserialize)]
pub struct AnalyticsDaysQuery {
    #[serde(default = "default_days")]
    pub days: i32,
}

fn default_days() -> i32 {
    30
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tenants", post(create_tenant).get(list_tenants))
        .route(
            "/tenants/:id",
            get(get_tenant).put(update_tenant).delete(delete_tenant),
        )
        .route("/tenants/:id/analytics", get(get_analytics))
        .route("/tenants/:id/usage", get(get_usage))
}

async fn create_tenant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTenantRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::bad_request(e.to_string()))?;
    let tenant = tenant_controller::create_tenant(&state, &req).await?;
    Ok((StatusCode::CREATED, Json(tenant)))
}

async fn list_tenants(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let tenants = tenant_controller::list_tenants(&state).await?;
    Ok((StatusCode::OK, Json(tenants)))
}

async fn get_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let tenant = tenant_controller::get_tenant(&state, id).await?;
    Ok((StatusCode::OK, Json(tenant)))
}

async fn update_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTenantRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate().map_err(|e| AppError::bad_request(e.to_string()))?;
    let tenant = tenant_controller::update_tenant(&state, id, &req).await?;
    Ok((StatusCode::OK, Json(tenant)))
}

async fn delete_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    tenant_controller::delete_tenant(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_analytics(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(q): Query<AnalyticsDaysQuery>,
) -> Result<impl IntoResponse, AppError> {
    let analytics = tenant_controller::get_tenant_analytics(&state, id, q.days).await?;
    Ok((StatusCode::OK, Json(analytics)))
}

async fn get_usage(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let usage = tenant_controller::get_tenant_usage(&state, id).await?;
    Ok((StatusCode::OK, Json(usage)))
}
