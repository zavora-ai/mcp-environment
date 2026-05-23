use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::*;

#[derive(Clone)]
pub struct EnvironmentServer {
    envs: Arc<RwLock<HashMap<String, Environment>>>,
    deploys: Arc<RwLock<Vec<Deployment>>>,
}

impl EnvironmentServer {
    pub fn new() -> Self {
        let mut envs = HashMap::new();
        let now = Utc::now();
        for (id, name, env_type, region) in [
            ("env_dev", "Development", EnvironmentType::Development, "local"),
            ("env_staging", "Staging", EnvironmentType::Staging, "us-east-1"),
            ("env_prod", "Production", EnvironmentType::Production, "us-east-1"),
            ("env_eu_prod", "EU Production", EnvironmentType::Regional, "eu-west-1"),
        ] {
            envs.insert(id.into(), Environment {
                environment_id: id.into(), name: name.into(), env_type, region: region.into(),
                status: EnvironmentStatus::Healthy, runtime_version: "adk-rust-0.12.4".into(),
                active_release_id: Some("rel_2026_0518_1842".into()), config_version: 1,
                config: serde_json::json!({"model_route": "gemini-3.1-flash", "memory_backend": "redis", "payment_mode": "production"}),
                worker_pools: vec![
                    WorkerPool { pool_id: format!("{}_model", id), pool_type: PoolType::Model, min_capacity: 2, desired_capacity: 4, max_capacity: 20, current_capacity: 4, status: "healthy".into() },
                    WorkerPool { pool_id: format!("{}_graph", id), pool_type: PoolType::Graph, min_capacity: 1, desired_capacity: 2, max_capacity: 10, current_capacity: 2, status: "healthy".into() },
                    WorkerPool { pool_id: format!("{}_browser", id), pool_type: PoolType::Browser, min_capacity: 2, desired_capacity: 5, max_capacity: 50, current_capacity: 5, status: "healthy".into() },
                    WorkerPool { pool_id: format!("{}_payment", id), pool_type: PoolType::Payment, min_capacity: 1, desired_capacity: 2, max_capacity: 8, current_capacity: 2, status: "healthy".into() },
                ],
                credential_refs: vec!["credref_anthropic_prod".into(), "credref_payments_signing_key".into()],
                protocol_bindings: serde_json::json!({"a2a": "enabled", "mcp": "enabled", "acp": "enabled"}),
                updated_at: now,
            });
        }
        Self { envs: Arc::new(RwLock::new(envs)), deploys: Arc::new(RwLock::new(Vec::new())) }
    }
}

// --- Inputs ---
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListEnvironmentsInput { #[serde(default)] pub status_filter: Option<EnvironmentStatus> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetEnvironmentInput { pub environment_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SyncConfigInput { pub environment_id: String, #[serde(default)] pub config: Option<serde_json::Value>, #[serde(default)] pub mode: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ValidateEnvironmentInput { pub environment_id: String, #[serde(default)] pub scope: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ScaleWorkerPoolInput { pub environment_id: String, pub pool_type: PoolType, pub desired_capacity: u32, #[serde(default)] pub reason: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PromoteBuildInput { pub release_bundle_id: String, pub target_environment_id: String, pub promoted_by: String, #[serde(default)] pub strategy: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RollbackDeployInput { pub environment_id: String, pub reason: String, #[serde(default)] pub reason_code: Option<String> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetDeployHistoryInput { pub environment_id: String, #[serde(default)] pub limit: Option<usize> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DetectConfigDriftInput { pub environment_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetWorkerPoolStatusInput { pub environment_id: String }

#[tool_router(server_handler)]
impl EnvironmentServer {
    #[tool(description = "Show dev, staging, production, EU production, tenant envs")]
    async fn list_environments(&self, Parameters(i): Parameters<ListEnvironmentsInput>) -> String {
        let envs = self.envs.read().await;
        let summary: Vec<_> = envs.values()
            .filter(|e| i.status_filter.as_ref().is_none_or(|s| &e.status == s))
            .map(|e| {
                let workers: String = e.worker_pools.iter().map(|w| format!("{:?}:{}/{}", w.pool_type, w.current_capacity, w.desired_capacity)).collect::<Vec<_>>().join(", ");
                serde_json::json!({
                    "environment_id": e.environment_id, "name": e.name, "type": e.env_type,
                    "region": e.region, "status": e.status, "runtime_version": e.runtime_version,
                    "active_release_id": e.active_release_id, "workers": workers,
                })
            }).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Inspect selected environment configuration")]
    async fn get_environment(&self, Parameters(i): Parameters<GetEnvironmentInput>) -> String {
        match self.envs.read().await.get(&i.environment_id) {
            Some(env) => serde_json::to_string_pretty(env).unwrap(),
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Sync secrets, policies, providers, memory, MCP bindings (plan/apply)")]
    async fn sync_environment_config(&self, Parameters(i): Parameters<SyncConfigInput>) -> String {
        let mode = i.mode.unwrap_or_else(|| "plan".into());
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.environment_id) {
            Some(env) => {
                if mode == "apply" {
                    if let Some(config) = i.config {
                        env.config = config;
                        env.config_version += 1;
                        env.updated_at = Utc::now();
                    }
                    serde_json::to_string_pretty(&serde_json::json!({"environment_id": i.environment_id, "mode": "apply", "config_version": env.config_version, "status": "applied"})).unwrap()
                } else {
                    serde_json::to_string_pretty(&serde_json::json!({"environment_id": i.environment_id, "mode": "plan", "current_config_version": env.config_version, "changes_pending": i.config.is_some(), "requires_approval": env.env_type == EnvironmentType::Production})).unwrap()
                }
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Run provider, memory, protocol, payment, and policy checks")]
    async fn validate_environment(&self, Parameters(i): Parameters<ValidateEnvironmentInput>) -> String {
        let envs = self.envs.read().await;
        match envs.get(&i.environment_id) {
            Some(env) => {
                let workers_healthy = env.worker_pools.iter().all(|w| w.current_capacity >= w.min_capacity);
                let checks = vec![
                    Check { name: "provider_connectivity".into(), status: "passing".into(), message: "Model routes reachable".into() },
                    Check { name: "memory_backend".into(), status: "passing".into(), message: "Redis/Postgres connected".into() },
                    Check { name: "worker_health".into(), status: if workers_healthy { "passing" } else { "failing" }.into(), message: format!("{} pools", env.worker_pools.len()) },
                    Check { name: "protocol_bindings".into(), status: "passing".into(), message: "A2A/MCP/ACP enabled".into() },
                    Check { name: "credential_bindings".into(), status: "passing".into(), message: format!("{} refs valid", env.credential_refs.len()) },
                    Check { name: "config_schema".into(), status: "passing".into(), message: "Configuration valid".into() },
                    Check { name: "rollback_readiness".into(), status: "passing".into(), message: "Previous release available".into() },
                ];
                let passing = checks.iter().filter(|c| c.status == "passing").count() as u32;
                let failing = checks.iter().filter(|c| c.status == "failing").count() as u32;
                let review = checks.iter().filter(|c| c.status == "review").count() as u32;
                let result = ValidationResult {
                    validation_id: format!("val_{}", Uuid::new_v4().simple()),
                    environment_id: i.environment_id, scope: i.scope.unwrap_or_else(|| "full".into()),
                    checks, passing, failing, review,
                    promotion_allowed: failing == 0, validated_at: Utc::now(),
                };
                serde_json::to_string_pretty(&result).unwrap()
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Scale model, graph, browser, code, realtime, payment workers")]
    async fn scale_worker_pool(&self, Parameters(i): Parameters<ScaleWorkerPoolInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.environment_id) {
            Some(env) => {
                if let Some(pool) = env.worker_pools.iter_mut().find(|p| p.pool_type == i.pool_type) {
                    let prev = pool.desired_capacity;
                    pool.desired_capacity = i.desired_capacity.clamp(pool.min_capacity, pool.max_capacity);
                    pool.current_capacity = pool.desired_capacity;
                    env.updated_at = Utc::now();
                    serde_json::to_string_pretty(&serde_json::json!({
                        "environment_id": i.environment_id, "pool_type": i.pool_type,
                        "previous_capacity": prev, "target_capacity": pool.desired_capacity,
                        "reason": i.reason.unwrap_or_default(), "status": "scaled"
                    })).unwrap()
                } else {
                    format!("Pool type {:?} not found", i.pool_type)
                }
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Promote immutable release bundle to target environment")]
    async fn promote_build(&self, Parameters(i): Parameters<PromoteBuildInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.target_environment_id) {
            Some(env) => {
                let prev = env.active_release_id.clone();
                env.active_release_id = Some(i.release_bundle_id.clone());
                env.updated_at = Utc::now();
                let deploy = Deployment {
                    deployment_id: format!("deploy_{}", Uuid::new_v4().simple()),
                    environment_id: i.target_environment_id.clone(),
                    release_bundle_id: i.release_bundle_id, previous_release_id: prev,
                    strategy: i.strategy.unwrap_or_else(|| "immediate".into()),
                    status: DeployStatus::Succeeded, promoted_by: i.promoted_by,
                    validation_id: None, approval_id: None,
                    created_at: Utc::now(), completed_at: Some(Utc::now()),
                };
                drop(envs);
                self.deploys.write().await.push(deploy.clone());
                serde_json::to_string_pretty(&deploy).unwrap()
            }
            None => format!("Environment not found: {}", i.target_environment_id),
        }
    }

    #[tool(description = "Roll back to previous release bundle")]
    async fn rollback_deploy(&self, Parameters(i): Parameters<RollbackDeployInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.environment_id) {
            Some(env) => {
                let deploys = self.deploys.read().await;
                let prev = deploys.iter().rev()
                    .filter(|d| d.environment_id == i.environment_id && d.status == DeployStatus::Succeeded)
                    .nth(1);
                match prev {
                    Some(prev_deploy) => {
                        let from = env.active_release_id.clone();
                        env.active_release_id = Some(prev_deploy.release_bundle_id.clone());
                        env.updated_at = Utc::now();
                        serde_json::to_string_pretty(&serde_json::json!({
                            "rollback_id": format!("rb_{}", Uuid::new_v4().simple()),
                            "environment_id": i.environment_id,
                            "from_release": from, "to_release": prev_deploy.release_bundle_id,
                            "reason": i.reason, "reason_code": i.reason_code,
                            "status": "rolled_back"
                        })).unwrap()
                    }
                    None => "No previous deployment to roll back to".into(),
                }
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Inspect release events and promotion evidence")]
    async fn get_deploy_history(&self, Parameters(i): Parameters<GetDeployHistoryInput>) -> String {
        let deploys = self.deploys.read().await;
        let history: Vec<_> = deploys.iter().rev()
            .filter(|d| d.environment_id == i.environment_id)
            .take(i.limit.unwrap_or(10)).cloned().collect();
        serde_json::to_string_pretty(&history).unwrap()
    }

    #[tool(description = "Detect configuration drift from expected state")]
    async fn detect_config_drift(&self, Parameters(i): Parameters<DetectConfigDriftInput>) -> String {
        let envs = self.envs.read().await;
        match envs.get(&i.environment_id) {
            Some(env) => {
                // Simulate drift detection
                let mut items = Vec::new();
                if env.config_version > 1 {
                    items.push(DriftItem { target: "config_version".into(), expected: "1".into(), actual: env.config_version.to_string(), severity: "low".into() });
                }
                let report = DriftReport {
                    drift_id: format!("drift_{}", Uuid::new_v4().simple()),
                    environment_id: i.environment_id,
                    total_drifted: items.len() as u32,
                    drifted_items: items,
                    detected_at: Utc::now(),
                };
                serde_json::to_string_pretty(&report).unwrap()
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }

    #[tool(description = "Get detailed worker pool status for an environment")]
    async fn get_worker_pool_status(&self, Parameters(i): Parameters<GetWorkerPoolStatusInput>) -> String {
        let envs = self.envs.read().await;
        match envs.get(&i.environment_id) {
            Some(env) => {
                let pools: Vec<_> = env.worker_pools.iter().map(|p| serde_json::json!({
                    "pool_id": p.pool_id, "pool_type": p.pool_type,
                    "min": p.min_capacity, "desired": p.desired_capacity,
                    "max": p.max_capacity, "current": p.current_capacity,
                    "status": p.status,
                    "utilization": format!("{}%", (p.current_capacity as f64 / p.desired_capacity.max(1) as f64 * 100.0) as u32),
                })).collect();
                serde_json::to_string_pretty(&serde_json::json!({
                    "environment_id": i.environment_id, "pools": pools, "total_pools": pools.len()
                })).unwrap()
            }
            None => format!("Environment not found: {}", i.environment_id),
        }
    }
}
