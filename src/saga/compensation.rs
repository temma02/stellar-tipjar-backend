use crate::errors::app_error::AppError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_name: String,
    pub status: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaExecution {
    pub id: Uuid,
    pub saga_type: String,
    pub status: String,
    pub steps: Vec<SagaStep>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait SagaCompensation: Send + Sync {
    async fn compensate(&self, step_data: serde_json::Value) -> Result<(), AppError>;
}

pub struct SagaOrchestrator {
    pool: PgPool,
    compensations: HashMap<String, Box<dyn SagaCompensation>>,
}

impl SagaOrchestrator {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            compensations: HashMap::new(),
        }
    }

    pub fn register_compensation(
        &mut self,
        step_name: String,
        handler: Box<dyn SagaCompensation>,
    ) {
        self.compensations.insert(step_name, handler);
    }

    pub async fn start_saga(&self, saga_type: &str) -> Result<Uuid, AppError> {
        let saga_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO saga_executions (id, saga_type, status, created_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(saga_id)
        .bind(saga_type)
        .bind("running")
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(saga_id)
    }

    pub async fn record_step(
        &self,
        saga_id: Uuid,
        step_name: &str,
        input: serde_json::Value,
        output: Option<serde_json::Value>,
        error: Option<String>,
    ) -> Result<(), AppError> {
        let step_id = Uuid::new_v4();
        let now = Utc::now();
        let status = if error.is_some() { "failed" } else { "completed" };

        sqlx::query(
            r#"
            INSERT INTO saga_steps (id, saga_id, step_name, status, input, output, error, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(step_id)
        .bind(saga_id)
        .bind(step_name)
        .bind(status)
        .bind(input)
        .bind(output)
        .bind(error)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(())
    }

    pub async fn compensate_saga(&self, saga_id: Uuid) -> Result<(), AppError> {
        let steps: Vec<SagaStep> = sqlx::query_as(
            "SELECT id, saga_id, step_name, status, input, output, error, created_at FROM saga_steps WHERE saga_id = $1 ORDER BY created_at DESC"
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        for step in steps {
            if let Some(handler) = self.compensations.get(&step.step_name) {
                handler.compensate(step.input).await?;
            }
        }

        sqlx::query("UPDATE saga_executions SET status = 'compensated' WHERE id = $1")
            .bind(saga_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(())
    }

    pub async fn complete_saga(&self, saga_id: Uuid) -> Result<(), AppError> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE saga_executions SET status = 'completed', completed_at = $1 WHERE id = $2",
        )
        .bind(now)
        .bind(saga_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(())
    }

    pub async fn get_saga(&self, saga_id: Uuid) -> Result<SagaExecution, AppError> {
        let execution: (Uuid, String, String, Option<DateTime<Utc>>, DateTime<Utc>) =
            sqlx::query_as(
                "SELECT id, saga_type, status, completed_at, created_at FROM saga_executions WHERE id = $1"
            )
            .bind(saga_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::not_found("Saga not found".to_string()))?;

        let steps: Vec<SagaStep> = sqlx::query_as(
            "SELECT id, saga_id, step_name, status, input, output, error, created_at FROM saga_steps WHERE saga_id = $1"
        )
        .bind(saga_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(SagaExecution {
            id: execution.0,
            saga_type: execution.1,
            status: execution.2,
            steps,
            created_at: execution.4,
            completed_at: execution.3,
        })
    }
}
