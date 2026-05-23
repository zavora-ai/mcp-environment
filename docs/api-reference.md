# API Reference

## list_environments

List all environments with status and worker summary.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `status_filter` | enum | No | Filter by: `healthy`, `degraded`, `review`, `disabled`, `maintenance` |

**Returns:** Array of environment summaries (ID, name, type, region, status, release, workers).

---

## get_environment

Inspect full environment configuration including worker pools, credential refs, and protocol bindings.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to inspect |

**Returns:** Full `Environment` object with config, worker pools, credential refs, protocol bindings.

---

## sync_environment_config

Sync configuration with a plan/apply safety model. Production requires approval.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Target environment |
| `config` | object | No | New configuration to apply |
| `mode` | string | No | `plan` (default) or `apply` |

**Plan mode:** Shows what would change and whether approval is required.
**Apply mode:** Applies changes and increments config version.

---

## validate_environment

Run comprehensive validation checks before promotion or on schedule.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to validate |
| `scope` | string | No | `pre_promote`, `scheduled_health`, `incident_check`, `full` |

**Checks:** provider connectivity, memory backend, worker health, protocol bindings, credential bindings, config schema, rollback readiness.

**Returns:** `ValidationResult` with pass/fail/review counts and `promotion_allowed` flag.

---

## scale_worker_pool

Scale a specific worker pool type. Respects min/max capacity bounds.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Target environment |
| `pool_type` | enum | Yes | `model`, `graph`, `browser`, `code`, `realtime`, `payment`, `mcp`, `a2a`, `acp` |
| `desired_capacity` | integer | Yes | Target capacity (clamped to min/max) |
| `reason` | string | No | Why scaling is needed |

**Returns:** Previous and new capacity with status.

---

## promote_build

Promote an immutable release bundle to a target environment.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `release_bundle_id` | string | Yes | Release to promote |
| `target_environment_id` | string | Yes | Where to deploy |
| `promoted_by` | string | Yes | Who initiated promotion |
| `strategy` | string | No | `immediate`, `canary`, `blue_green`, `rolling` |

**Returns:** `Deployment` record with previous release tracked for rollback.

---

## rollback_deploy

Roll back to the previous successful release.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to roll back |
| `reason` | string | Yes | Why rolling back |
| `reason_code` | string | No | `failed_health_check`, `policy_regression`, `payment_error`, `operator_requested` |

**Returns:** Rollback confirmation with from/to release IDs.

---

## get_deploy_history

Inspect release events and promotion evidence.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to query |
| `limit` | integer | No | Max events (default 10) |

**Returns:** Array of `Deployment` records, newest first.

---

## detect_config_drift

Detect configuration drift from expected state.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to check |

**Returns:** `DriftReport` with list of drifted items, severity, and expected vs actual values.

---

## get_worker_pool_status

Get detailed worker pool utilization and health for an environment.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `environment_id` | string | Yes | Environment to query |

**Returns:** All pools with min/desired/max/current capacity, utilization percentage, and health status.
