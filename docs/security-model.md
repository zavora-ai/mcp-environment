# Security Model

## Core Principle: Runtime Control Plane Safety

The Environment MCP manages where agents run and how builds move between environments. Unauthorized changes can cause outages, data loss, or cost spikes.

## High-Risk Operations

| Operation | Risk | Mitigation |
|-----------|------|-----------|
| `promote_build` to production | Outage, regression | Requires validation + approval |
| `rollback_deploy` | Session disruption | Audit reason required |
| `scale_worker_pool` (production) | Cost spike | Min/max bounds enforced |
| `sync_environment_config` (apply) | Config corruption | Plan/apply model, versioning |

## Access Control

| Action | Required Permission |
|--------|-------------------|
| `list_environments` | Any authenticated actor |
| `get_environment` | Environment read access |
| `validate_environment` | Environment read access |
| `get_deploy_history` | Environment read access |
| `detect_config_drift` | Environment read access |
| `get_worker_pool_status` | Environment read access |
| `sync_environment_config` | Environment admin + governance approval for production |
| `scale_worker_pool` | Runtime admin + policy check for production |
| `promote_build` | Release promoter + validation + approval |
| `rollback_deploy` | Incident operator or release admin |

## Design Decisions

- **No raw secrets** — only `credential_refs` pointing to Credentials Vault
- **No artifact binaries** — only `release_bundle_id` pointing to Artifact Store
- **Config versioning** — every sync increments version, enabling drift detection
- **Capacity bounds** — scaling is clamped to min/max, preventing runaway costs
- **Plan/apply** — production config changes require explicit apply after review
