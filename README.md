# taskd

Lightweight project management tool built for AI agent workflows. SQLite backend, Rust CLI and API server, terminal-themed web UI with live SSE updates.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (for the web UI)

## Install

```bash
git clone git@github.com:irelandmi/taskd.git
cd taskd

# Install the CLI and server binaries
cargo install --path crates/cli
cargo install --path crates/server

# Build the frontend
cd frontend && npm install && npm run build && cd ..
```

This installs `taskd` and `taskd-server` to `~/.cargo/bin/`. No external dependencies — SQLite is compiled from source.

## Quick Start

```bash
# Start the server with the web UI
taskd-server --port 3000 --static-dir frontend/dist

# Use the CLI
taskd project create "My Project"
taskd task create --project <id> "My first task"
taskd task list --project <id>
```

## Architecture

```
crates/
  core/       # SQLite schema, models, queries (taskd-core)
  cli/        # CLI binary (taskd)
  server/     # Axum API server (taskd-server)
frontend/     # TypeScript + Vite web UI
docs/         # Architecture and schema docs
tests/        # E2E shell tests
```

## CLI

```bash
# Projects
taskd project list
taskd project create <name> [--description <desc>]
taskd project show <id>
taskd project delete <id>

# Epics
taskd epic list --project <id>
taskd epic create --project <id> <name> [--description <desc>]
taskd epic show <id>
taskd epic close <id>
taskd epic delete <id>

# Tasks
taskd task list --project <id> [--status <s>] [--epic <id>] [--assignee <a>] [--label <l>] [--kind <k>]
taskd task create --project <id> <title> [--epic <id>] [--kind <k>] [--parent <id>] [--priority <p>] [--assignee <a>] [--label <l>]...
taskd task show <id>
taskd task update <id> [--title <t>] [--description <d>] [--status <s>] [--priority <p>] [--assignee <a>] [--epic <id>] [--kind <k>]
taskd task done <id>
taskd task delete <id>

# Task outputs (file paths, commit SHAs, URLs, free text)
taskd task output <id> --kind <kind> --ref <ref> [--label <label>]
taskd task outputs <id>

# Task dependencies
taskd task block <id> --by <dep_id>
taskd task unblock <id> --from <dep_id>

# Labels
taskd label list
taskd label create <name> [--color <hex>]
taskd label delete <id>
```

IDs are human-readable (`bold-fox-a3f1`) and support prefix lookup (`bold-fox`).

## API

The server exposes a REST API at `/api`. All endpoints return JSON.

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/projects | List projects |
| POST | /api/projects | Create project |
| GET | /api/projects/:id | Get project |
| PATCH | /api/projects/:id | Update project |
| DELETE | /api/projects/:id | Delete project |
| GET | /api/projects/:pid/epics | List epics |
| POST | /api/projects/:pid/epics | Create epic |
| GET | /api/epics/:id | Get epic |
| PATCH | /api/epics/:id | Update epic |
| DELETE | /api/epics/:id | Delete epic |
| GET | /api/projects/:pid/tasks | List tasks (with query filters) |
| POST | /api/projects/:pid/tasks | Create task |
| GET | /api/tasks/:id | Get task (includes children, outputs, dependencies) |
| PATCH | /api/tasks/:id | Update task |
| DELETE | /api/tasks/:id | Delete task |
| PUT | /api/tasks/:id/labels | Set task labels |
| GET | /api/tasks/:id/outputs | List task outputs |
| POST | /api/tasks/:id/outputs | Add task output |
| POST | /api/tasks/:id/dependencies | Add dependency |
| DELETE | /api/tasks/:id/dependencies/:dep_id | Remove dependency |
| GET | /api/tasks/:id/events | List activity events |
| POST | /api/tasks/:id/events | Add comment |
| GET | /api/labels | List labels |
| POST | /api/labels | Create label |
| DELETE | /api/labels/:id | Delete label |
| GET | /api/events | SSE event stream |

## Data Model

- **Projects** contain **epics** (every project gets a Backlog epic automatically)
- **Epics** contain **tasks** (tasks default to Backlog when no epic is specified)
- **Tasks** have a type (`story`, `task`, `spike`, `bug`, `chore`), status (`todo`, `in_progress`, `done`, `cancelled`, `blocked`), priority, assignee, and labels
- **Tasks** can have sub-tasks (one level of nesting via `parent_id`)
- **Task outputs** are lightweight references to artifacts: file paths, commit SHAs, URLs, or free text
- **Task dependencies** form a directed acyclic graph — circular dependencies are rejected at insert time

## Web UI

Terminal-themed kanban board with:

- Kanban and timeline views per project
- Five status columns: To Do, In Progress, Done, Cancelled, Blocked
- Task detail page with inline editing, sub-task creation, outputs, dependencies, and activity log
- Live updates via Server-Sent Events

## Testing

```bash
# Unit tests (21 tests)
cargo test --workspace

# E2E CLI tests (35 tests)
bash tests/cli_e2e.sh
```

## Docs

- [Database schema](docs/database.md) - tables, indexes, migrations, cascade behavior
- [Agent architecture](docs/agent-architecture.md) - planner/executor/coordinator roles
- [Open questions](docs/open-questions.md) - unresolved design decisions
