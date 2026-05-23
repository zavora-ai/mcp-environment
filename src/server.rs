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
        for (id, name, tier) in [
            ("env_dev", "Development", "development"),
            ("env_staging", "Staging", "staging"),
            ("env_prod", "Production", "production"),
            ("env_eu_prod", "EU Production", "eu_production"),
        ] {
            envs.insert(id.into(), Environment {
                env_id: id.into(), name: name.into(), tier: tier.into(),
                status: EnvironmentStatus::Active,
                config: serde_json::json!({"model_route": "gemini-3.1-flash", "memory_backend": "redis"}),
                worker_pools: vec![
                    WorkerPool { pool_id: format!("{}_model", id), pool_type: "model".into(), desired: 3, running: 3, status: "healthy".into() },
                    WorkerPool { pool_id: format!("{}_graph", id), pool_type: "graph".into(), desired: 2, running: 2, status: "healthy".into() },
                ],
                current_build: Some("build_v1.8.3".into()),
                updated_at: now,
            });
        }
        Self { envs: Arc::new(RwLock::new(envs)), deploys: Arc::new(RwLock::new(Vec::new())) }
    }
}

// --- Inputs ---
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListEnvironmentsInput {}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetEnvironmentInput { pub env_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SyncConfigInput { pub env_id: String, #[serde(default)] pub config: Option<serde_json::Value> }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ValidateEnvironmentInput { pub env_id: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ScaleWorkerPoolInput { pub env_id: String, pub pool_type: String, pub desired: u32 }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PromoteBuildInput { pub build_id: String, pub target_env_id: String, pub promoted_by: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct RollbackDeployInput { pub env_id: String, pub reason: String }

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetDeployHistoryInput { pub env_id: String, #[serde(default)] pub limit: Option<usize> }

#[tool_router(server_handler)]
impl EnvironmentServer {
    #[tool(description = "Show dev, staging, production, EU production, tenant envs")]
    async fn list_environments(&self, Parameters(_): Parameters<ListEnvironmentsInput>) -> String {
        let envs = self.envs.read().await;
        let summary: Vec<_> = envs.values().map(|e| serde_json::json!({
            "env_id": e.env_id, "name": e.name, "tier": e.tier,
            "status": e.status, "current_build": e.current_build,
        })).collect();
        serde_json::to_string_pretty(&summary).unwrap()
    }

    #[tool(description = "Inspect selected environment configuration")]
    async fn get_environment(&self, Parameters(i): Parameters<GetEnvironmentInput>) -> String {
        match self.envs.read().await.get(&i.env_id) {
            Some(env) => serde_json::to_string_pretty(env).unwrap(),
            None => format!("Environment not found: {}", i.env_id),
        }
    }

    #[tool(description = "Sync secrets, policies, providers, memory, MCP bindings")]
    async fn sync_environment_config(&self, Parameters(i): Parameters<SyncConfigInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.env_id) {
            Some(env) => {
                if let Some(config) = i.config {
                    env.config = config;
                }
                env.updated_at = Utc::now();
                serde_json::to_string_pretty(&serde_json::json!({"env_id": i.env_id, "status": "synced"})).unwrap()
            }
            None => format!("Environment not found: {}", i.env_id),
        }
    }

    #[tool(description = "Run provider, memory, protocol, payment, and policy checks")]
    async fn validate_environment(&self, Parameters(i): Parameters<ValidateEnvironmentInput>) -> String {
        let envs = self.envs.read().await;
        match envs.get(&i.env_id) {
            Some(env) => {
                let checks = vec![
                    Check { name: "model_provider".into(), passed: true, message: "Model route reachable".into() },
                    Check { name: "memory_backend".into(), passed: true, message: "Redis connected".into() },
                    Check { name: "worker_health".into(), passed: env.worker_pools.iter().all(|w| w.running >= w.desired), message: format!("{} pools healthy", env.worker_pools.len()) },
                    Check { name: "config_valid".into(), passed: true, message: "Configuration schema valid".into() },
                ];
                let all_passed = checks.iter().all(|c| c.passed);
                let result = ValidationResult { env_id: i.env_id, checks, all_passed, validated_at: Utc::now() };
                serde_json::to_string_pretty(&result).unwrap()
            }
            None => format!("Environment not found: {}", i.env_id),
        }
    }

    #[tool(description = "Scale model, graph, browser, code, realtime, payment workers")]
    async fn scale_worker_pool(&self, Parameters(i): Parameters<ScaleWorkerPoolInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.env_id) {
            Some(env) => {
                if let Some(pool) = env.worker_pools.iter_mut().find(|p| p.pool_type == i.pool_type) {
                    pool.desired = i.desired;
                    pool.running = i.desired; // simulate instant scale
                    env.updated_at = Utc::now();
                    serde_json::to_string_pretty(&serde_json::json!({
                        "env_id": i.env_id, "pool_type": i.pool_type, "desired": i.desired, "status": "scaled"
                    })).unwrap()
                } else {
                    format!("Pool type '{}' not found in {}", i.pool_type, i.env_id)
                }
            }
            None => format!("Environment not found: {}", i.env_id),
        }
    }

    #[tool(description = "Promote tested build to target environment")]
    async fn promote_build(&self, Parameters(i): Parameters<PromoteBuildInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.target_env_id) {
            Some(env) => {
                let prev_build = env.current_build.clone();
                env.current_build = Some(i.build_id.clone());
                env.updated_at = Utc::now();
                let deploy = Deployment {
                    deploy_id: format!("dep_{}", Uuid::new_v4().simple()),
                    env_id: i.target_env_id.clone(), build_id: i.build_id,
                    status: DeployStatus::Succeeded, promoted_by: i.promoted_by,
                    promoted_from: prev_build, created_at: Utc::now(), completed_at: Some(Utc::now()),
                };
                drop(envs);
                self.deploys.write().await.push(deploy.clone());
                serde_json::to_string_pretty(&deploy).unwrap()
            }
            None => format!("Environment not found: {}", i.target_env_id),
        }
    }

    #[tool(description = "Roll back to previous release bundle")]
    async fn rollback_deploy(&self, Parameters(i): Parameters<RollbackDeployInput>) -> String {
        let mut envs = self.envs.write().await;
        match envs.get_mut(&i.env_id) {
            Some(env) => {
                let deploys = self.deploys.read().await;
                let prev = deploys.iter().rev()
                    .filter(|d| d.env_id == i.env_id && d.status == DeployStatus::Succeeded)
                    .nth(1);
                match prev {
                    Some(prev_deploy) => {
                        env.current_build = Some(prev_deploy.build_id.clone());
                        env.updated_at = Utc::now();
                        serde_json::to_string_pretty(&serde_json::json!({
                            "env_id": i.env_id, "rolled_back_to": prev_deploy.build_id,
                            "reason": i.reason, "status": "rolled_back"
                        })).unwrap()
                    }
                    None => "No previous deployment to roll back to".into(),
                }
            }
            None => format!("Environment not found: {}", i.env_id),
        }
    }

    #[tool(description = "Inspect release events and promotion evidence")]
    async fn get_deploy_history(&self, Parameters(i): Parameters<GetDeployHistoryInput>) -> String {
        let deploys = self.deploys.read().await;
        let history: Vec<_> = deploys.iter().rev()
            .filter(|d| d.env_id == i.env_id)
            .take(i.limit.unwrap_or(10))
            .collect();
        serde_json::to_string_pretty(&history).unwrap()
    }
}
