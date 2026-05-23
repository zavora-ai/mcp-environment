use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentStatus { Active, Degraded, Maintenance, Offline }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus { Pending, InProgress, Succeeded, Failed, RolledBack }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub env_id: String,
    pub name: String,
    pub tier: String, // development, staging, production, eu_production
    pub status: EnvironmentStatus,
    pub config: serde_json::Value,
    pub worker_pools: Vec<WorkerPool>,
    pub current_build: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPool {
    pub pool_id: String,
    pub pool_type: String, // model, graph, browser, code, realtime, payment
    pub desired: u32,
    pub running: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub deploy_id: String,
    pub env_id: String,
    pub build_id: String,
    pub status: DeployStatus,
    pub promoted_by: String,
    pub promoted_from: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub env_id: String,
    pub checks: Vec<Check>,
    pub all_passed: bool,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    pub message: String,
}
