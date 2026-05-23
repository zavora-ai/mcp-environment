use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentStatus { Healthy, Degraded, Review, Disabled, Maintenance }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentType { Development, Staging, Production, Regional, Tenant }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeployStatus { Pending, Canary, Running, Succeeded, Failed, RolledBack }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PoolType { Model, Graph, Browser, Code, Realtime, Payment, Mcp, A2a, Acp }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub environment_id: String,
    pub name: String,
    pub env_type: EnvironmentType,
    pub region: String,
    pub status: EnvironmentStatus,
    pub runtime_version: String,
    pub active_release_id: Option<String>,
    pub config_version: u32,
    pub config: serde_json::Value,
    pub worker_pools: Vec<WorkerPool>,
    pub credential_refs: Vec<String>,
    pub protocol_bindings: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPool {
    pub pool_id: String,
    pub pool_type: PoolType,
    pub min_capacity: u32,
    pub desired_capacity: u32,
    pub max_capacity: u32,
    pub current_capacity: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub deployment_id: String,
    pub environment_id: String,
    pub release_bundle_id: String,
    pub previous_release_id: Option<String>,
    pub strategy: String,
    pub status: DeployStatus,
    pub promoted_by: String,
    pub validation_id: Option<String>,
    pub approval_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub validation_id: String,
    pub environment_id: String,
    pub scope: String,
    pub checks: Vec<Check>,
    pub passing: u32,
    pub failing: u32,
    pub review: u32,
    pub promotion_allowed: bool,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub name: String,
    pub status: String, // passing, failing, review
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub drift_id: String,
    pub environment_id: String,
    pub drifted_items: Vec<DriftItem>,
    pub total_drifted: u32,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    pub target: String,
    pub expected: String,
    pub actual: String,
    pub severity: String,
}
