# Environment / Runtime MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-environment.svg)](https://crates.io/crates/mcp-environment)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Runtime environment management for [ADK-Rust Enterprise](https://enterprise.adk-rust.com). Environment registry, worker pools, deployments, validation, and promotion gates.

## Tools (8)

| Tool | Purpose |
|------|---------|
| `list_environments` | Show all environments (dev, staging, prod, EU) |
| `get_environment` | Inspect environment config and worker pools |
| `sync_environment_config` | Sync secrets, policies, providers, MCP bindings |
| `validate_environment` | Run provider, memory, worker, and config checks |
| `scale_worker_pool` | Scale model/graph/browser/code/realtime workers |
| `promote_build` | Promote tested build to target environment |
| `rollback_deploy` | Roll back to previous release |
| `get_deploy_history` | Inspect release events and promotion evidence |

## Installation

```json
{ "mcpServers": { "environment": { "command": "/path/to/mcp-environment" } } }
```

## Contributors

| [<img src="https://github.com/jkmaina.png" width="80px;"/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|

## License

Apache-2.0 — Part of [ADK-Rust Enterprise](https://enterprise.adk-rust.com). Built with ❤️ by [Zavora AI](https://zavora.ai)
