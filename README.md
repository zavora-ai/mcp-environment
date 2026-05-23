# Environment / Runtime MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-environment.svg)](https://crates.io/crates/mcp-environment)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Runtime control plane for [ADK-Rust Enterprise](https://enterprise.adk-rust.com). Manages runtime environments, worker pools, deployment bundles, configuration sync, provider routing, validation gates, promotion, rollback, and configuration drift detection.

## Purpose

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-environment/main/docs/architecture.svg" alt="Environment / Runtime MCP Architecture" width="800"/>
</p>

> Policy-controlled management of ADK-Rust runtime environments, worker pools, deployment bundles, configuration sync, provider routes, memory backend bindings, protocol bindings, payment runtime modes, validation gates, promotion, rollback, and configuration drift detection.

This MCP is the **runtime control plane** — it knows where agents run, what version they're on, and how builds move between environments.

## What This MCP Owns

```
✅ Environment registry          ✅ Worker pool scaling
✅ Runtime configuration          ✅ Deployment targets
✅ Release bundle promotion       ✅ Rollback coordination
✅ Provider routing config        ✅ Memory backend bindings
✅ Protocol bindings (A2A/MCP/ACP)✅ Payment mode bindings
✅ Validation status              ✅ Configuration drift detection
✅ Deploy history                 ✅ Config versioning (plan/apply)
```

## What This MCP Does NOT Own

```
❌ Raw secrets → Credentials Vault MCP
❌ Artifact binaries → Artifact Store MCP
❌ Policy authoring → Governance Policy MCP
❌ Payment execution → ADK-Payments MCP
❌ Raw traces/metrics → Observability stack
❌ Source repositories → GitHub MCP
❌ Session state → Session Memory MCP
```

## Tools (10)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_environments` | Show all environments with status and worker summary | Read-only |
| `get_environment` | Inspect full environment config, pools, bindings | Read-only |
| `sync_environment_config` | Sync config with plan/apply safety model | Internal write |
| `validate_environment` | Run provider, memory, protocol, payment, worker checks | Read-only |
| `scale_worker_pool` | Scale model/graph/browser/code/realtime/payment workers | Internal write |
| `promote_build` | Promote immutable release bundle to target environment | Production deploy |
| `rollback_deploy` | Roll back to previous release with audit reason | Production deploy |
| `get_deploy_history` | Inspect release events and promotion evidence | Read-only |
| `detect_config_drift` | Detect drift from expected configuration state | Read-only |
| `get_worker_pool_status` | Detailed worker pool utilization and health | Read-only |

## Example Prompts & Outputs

### List all environments

**Prompt:** "Show me all our environments"

```json
[
  {
    "environment_id": "env_prod",
    "name": "Production",
    "type": "production",
    "region": "us-east-1",
    "status": "healthy",
    "runtime_version": "adk-rust-0.12.4",
    "active_release_id": "rel_2026_0518_1842",
    "workers": "Model:4/4, Graph:2/2, Browser:5/5, Payment:2/2"
  },
  {
    "environment_id": "env_staging",
    "name": "Staging",
    "type": "staging",
    "region": "us-east-1",
    "status": "healthy",
    "active_release_id": "rel_2026_0518_1842"
  }
]
```

---

### Validate before promotion

**Prompt:** "Is production ready for a new deployment?"

**Tool:** `validate_environment`
```json
{ "environment_id": "env_prod", "scope": "pre_promote" }
```

**Output:**
```json
{
  "validation_id": "val_91af2b...",
  "environment_id": "env_prod",
  "scope": "pre_promote",
  "checks": [
    { "name": "provider_connectivity", "status": "passing", "message": "Model routes reachable" },
    { "name": "memory_backend", "status": "passing", "message": "Redis/Postgres connected" },
    { "name": "worker_health", "status": "passing", "message": "4 pools" },
    { "name": "protocol_bindings", "status": "passing", "message": "A2A/MCP/ACP enabled" },
    { "name": "credential_bindings", "status": "passing", "message": "2 refs valid" },
    { "name": "config_schema", "status": "passing", "message": "Configuration valid" },
    { "name": "rollback_readiness", "status": "passing", "message": "Previous release available" }
  ],
  "passing": 7,
  "failing": 0,
  "review": 0,
  "promotion_allowed": true
}
```

---

### Promote a release

**Prompt:** "Promote the staging build to production"

**Tool:** `promote_build`
```json
{
  "release_bundle_id": "rel_2026_0519_0921",
  "target_environment_id": "env_prod",
  "promoted_by": "ci_pipeline",
  "strategy": "canary"
}
```

**Output:**
```json
{
  "deployment_id": "deploy_fffbd202...",
  "environment_id": "env_prod",
  "release_bundle_id": "rel_2026_0519_0921",
  "previous_release_id": "rel_2026_0518_1842",
  "strategy": "canary",
  "status": "succeeded",
  "promoted_by": "ci_pipeline"
}
```

---

### Scale workers for traffic spike

**Prompt:** "Scale browser workers to 20 in production — checkout traffic is spiking"

**Tool:** `scale_worker_pool`
```json
{
  "environment_id": "env_prod",
  "pool_type": "browser",
  "desired_capacity": 20,
  "reason": "checkout traffic spike"
}
```

**Output:**
```json
{
  "environment_id": "env_prod",
  "pool_type": "browser",
  "previous_capacity": 5,
  "target_capacity": 20,
  "reason": "checkout traffic spike",
  "status": "scaled"
}
```

---

### Rollback a failed deployment

**Prompt:** "Roll back production — payment validation is failing"

**Tool:** `rollback_deploy`
```json
{
  "environment_id": "env_prod",
  "reason": "Payment validation failing in canary",
  "reason_code": "payment_error"
}
```

**Output:**
```json
{
  "rollback_id": "rb_448a2b...",
  "environment_id": "env_prod",
  "from_release": "rel_2026_0519_0921",
  "to_release": "rel_2026_0518_1842",
  "reason": "Payment validation failing in canary",
  "reason_code": "payment_error",
  "status": "rolled_back"
}
```

---

### Sync config with plan/apply

**Prompt:** "Plan a config sync for production"

**Tool:** `sync_environment_config`
```json
{ "environment_id": "env_prod", "mode": "plan" }
```

**Output:**
```json
{
  "environment_id": "env_prod",
  "mode": "plan",
  "current_config_version": 1,
  "changes_pending": false,
  "requires_approval": true
}
```

---

### Detect configuration drift

**Prompt:** "Check if production has drifted from expected state"

**Tool:** `detect_config_drift`
```json
{ "environment_id": "env_prod" }
```

**Output:**
```json
{
  "drift_id": "drift_7c3a...",
  "environment_id": "env_prod",
  "total_drifted": 0,
  "drifted_items": []
}
```

---

### Worker pool status

**Prompt:** "Show me worker utilization in production"

**Tool:** `get_worker_pool_status`
```json
{ "environment_id": "env_prod" }
```

**Output:**
```json
{
  "environment_id": "env_prod",
  "total_pools": 4,
  "pools": [
    { "pool_type": "model", "desired": 4, "current": 4, "utilization": "100%", "status": "healthy" },
    { "pool_type": "graph", "desired": 2, "current": 2, "utilization": "100%", "status": "healthy" },
    { "pool_type": "browser", "desired": 5, "current": 5, "utilization": "100%", "status": "healthy" },
    { "pool_type": "payment", "desired": 2, "current": 2, "utilization": "100%", "status": "healthy" }
  ]
}
```

## Worker Pool Types

| Pool | Purpose |
|------|---------|
| `model` | LLM inference workers |
| `graph` | Workflow graph execution |
| `browser` | Browser automation / Playwright |
| `code` | Code execution sandboxes |
| `realtime` | Voice/streaming sessions |
| `payment` | Payment processing workers |
| `mcp` | MCP server workers |
| `a2a` | Agent-to-Agent protocol workers |
| `acp` | ACP coding delegate workers |

## Environment Types

| Type | Description |
|------|-------------|
| `development` | Local dev, no restrictions |
| `staging` | Pre-production testing |
| `production` | Live traffic, full governance |
| `regional` | Region-specific production (EU, APAC) |
| `tenant` | Tenant-isolated environments |

## Integration with Other MCPs

| MCP | Relationship |
|-----|-------------|
| **Credentials Vault** | Stores `credential_refs` — never raw secrets |
| **Artifact Store** | Release bundles stored as immutable artifacts |
| **Governance Policy** | Promotion/scaling require policy evaluation |
| **Session Memory** | Rollback must handle active sessions |
| **ADK-Payments** | Payment worker scaling and signing key bindings |

## Installation

```bash
git clone https://github.com/zavora-ai/mcp-environment
cd mcp-environment
cargo build --release
```

### MCP Client Config

```json
{
  "mcpServers": {
    "environment": {
      "command": "/path/to/mcp-environment"
    }
  }
}
```

Works with Claude Desktop, Kiro, Codex, Cursor, Windsurf, Antigravity, and Open Code.

## Contributing

PRs welcome. Run `cargo clippy` and `cargo fmt` before submitting.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
