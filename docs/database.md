# Database

SQLite database using WAL journal mode with foreign keys enforced. Schema is versioned via `PRAGMA user_version` with incremental migrations.

## ID Format

All primary keys use human-readable names in the format `{adjective}-{noun}-{4 hex}`, e.g. `bold-fox-a3f1`. Generated from a pool of 144 adjectives, 198 nouns, and a 16-bit hex suffix (~1.87 billion combinations). An atomic counter ensures uniqueness even when IDs are generated within the same nanosecond.

Lookups support prefix matching — `bold-fox` will match `bold-fox-a3f1`. If a prefix is ambiguous (matches multiple entities), the lookup returns a conflict error rather than silently picking one.

## Schema

### projects

Top-level container for all work.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | Human-readable ID |
| name | TEXT | NOT NULL | Project name |
| description | TEXT | NOT NULL DEFAULT '' | Optional description |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp |

Creating a project auto-creates a **Backlog** epic for it (within a transaction).

### epics

Grouping of tasks within a project. Every project has at least one epic (Backlog).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | Human-readable ID |
| project_id | TEXT | NOT NULL, FK → projects(id) ON DELETE CASCADE | Parent project |
| name | TEXT | NOT NULL | Epic name (e.g. "Backlog", "Auth") |
| description | TEXT | NOT NULL DEFAULT '' | Optional description |
| status | TEXT | NOT NULL DEFAULT 'open', CHECK IN ('open', 'closed') | Epic status |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:** `idx_epics_project` on `project_id`

### tasks

Work items. Can be nested one level (parent → children).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | Human-readable ID |
| epic_id | TEXT | NOT NULL, FK → epics(id) ON DELETE CASCADE | Parent epic |
| project_id | TEXT | NOT NULL, FK → projects(id) ON DELETE CASCADE | Parent project |
| parent_id | TEXT | NULLABLE, FK → tasks(id) ON DELETE CASCADE | Parent task (for sub-tasks) |
| kind | TEXT | NOT NULL DEFAULT 'task', CHECK IN ('story', 'task', 'spike', 'bug', 'chore') | Work item type |
| title | TEXT | NOT NULL | Task title |
| description | TEXT | NOT NULL DEFAULT '' | Optional description |
| status | TEXT | NOT NULL DEFAULT 'todo', CHECK IN ('todo', 'in_progress', 'done', 'cancelled', 'blocked') | Task status |
| priority | TEXT | NOT NULL DEFAULT 'medium', CHECK IN ('low', 'medium', 'high', 'urgent') | Priority level |
| assignee | TEXT | NULLABLE | Assigned user |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp |
| updated_at | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:** `idx_tasks_project` on `project_id`, `idx_tasks_epic` on `epic_id`, `idx_tasks_status` on `status`, `idx_tasks_parent` on `parent_id`, `idx_tasks_kind` on `kind`

When `epic_id` is omitted during creation, the task is auto-assigned to the project's Backlog epic.

### labels

Global labels that can be applied to any task.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | Human-readable ID |
| name | TEXT | NOT NULL, UNIQUE | Label name |
| color | TEXT | NOT NULL DEFAULT '#6b7280' | Hex color |

### task_labels

Join table linking tasks to labels. Many-to-many.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| task_id | TEXT | NOT NULL, FK → tasks(id) ON DELETE CASCADE | Task reference |
| label_id | TEXT | NOT NULL, FK → labels(id) ON DELETE CASCADE | Label reference |

**Primary key:** (task_id, label_id)

### task_outputs

Lightweight references to outputs produced by a task (file paths, commit SHAs, URLs, free text).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| id | TEXT | PRIMARY KEY | Human-readable ID |
| task_id | TEXT | NOT NULL, FK → tasks(id) ON DELETE CASCADE | Parent task |
| kind | TEXT | NOT NULL, CHECK IN ('file', 'commit', 'url', 'text') | Output type |
| reference | TEXT | NOT NULL | File path, commit SHA, URL, or free text |
| label | TEXT | NOT NULL DEFAULT '' | Optional human label |
| created_at | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:** `idx_task_outputs_task` on `task_id`

### task_dependencies

Directed dependency graph between tasks. A task depends on another task being done before it is ready.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| task_id | TEXT | NOT NULL, FK → tasks(id) ON DELETE CASCADE | The blocked task |
| depends_on | TEXT | NOT NULL, FK → tasks(id) ON DELETE CASCADE | The blocking task |

**Primary key:** (task_id, depends_on)

Cycle detection is enforced on insert — circular dependencies are rejected.

## Cascade Behavior

Deleting a parent cascades to all children:

- Delete **project** → deletes its epics and tasks
- Delete **epic** → deletes its tasks
- Delete **task** → deletes its sub-tasks (children)
- Delete **task** → deletes its outputs and dependency links
- Delete **label** → removes it from all task_labels entries

## Relationships

```
projects 1──* epics 1──* tasks *──* labels
                          │
                          ├──* tasks (sub-tasks via parent_id)
                          ├──* task_outputs
                          └──* task_dependencies (task_id ──► depends_on)
```

## Transactions

Multi-statement mutations are wrapped in explicit transactions with rollback on failure:

- **create_project** — inserts project + Backlog epic atomically
- **create_task** — inserts task + label associations atomically
- **set_task_labels** — deletes existing labels + inserts new ones atomically

## Configuration

| Setting | Value |
|---------|-------|
| Journal mode | WAL |
| Foreign keys | ON |
| Schema version | Tracked via `PRAGMA user_version` (currently 3) |
| Default file | `taskd.db` |
| Migrations | Incremental `if version < N` chain in `db.rs` |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/projects | List all projects |
| POST | /api/projects | Create a project (auto-creates Backlog epic) |
| GET | /api/projects/:id | Get a project |
| PATCH | /api/projects/:id | Update a project |
| DELETE | /api/projects/:id | Delete a project (cascades) |
| GET | /api/projects/:pid/epics | List epics for a project |
| POST | /api/projects/:pid/epics | Create an epic |
| GET | /api/epics/:id | Get an epic |
| PATCH | /api/epics/:id | Update an epic |
| DELETE | /api/epics/:id | Delete an epic (cascades) |
| GET | /api/projects/:pid/tasks | List tasks (supports query filters) |
| POST | /api/projects/:pid/tasks | Create a task (epic_id optional, defaults to Backlog) |
| GET | /api/tasks/:id | Get a task (includes children) |
| PATCH | /api/tasks/:id | Update a task |
| DELETE | /api/tasks/:id | Delete a task (cascades to sub-tasks) |
| PUT | /api/tasks/:id/labels | Set labels on a task |
| GET | /api/labels | List all labels |
| POST | /api/labels | Create a label |
| DELETE | /api/labels/:id | Delete a label |
| GET | /api/tasks/:id/outputs | List outputs for a task |
| POST | /api/tasks/:id/outputs | Attach an output to a task |
| POST | /api/tasks/:id/dependencies | Add a dependency |
| DELETE | /api/tasks/:id/dependencies/:dep_id | Remove a dependency |

### Task List Filters

`GET /api/projects/:pid/tasks` accepts query parameters:

| Parameter | Description |
|-----------|-------------|
| status | Filter by status (todo, in_progress, done, cancelled, blocked) |
| epic_id | Filter by epic |
| assignee | Filter by assignee |
| label | Filter by label name |
| kind | Filter by type (story, task, spike, bug, chore) |
| parent_id | Filter by parent task |

## CLI

Binary: `taskd`. Use `--db <path>` to specify the database file (default: `taskd.db`).

### Commands

```
taskd project list
taskd project create <name> [--description <desc>]
taskd project show <id>
taskd project delete <id>

taskd epic list --project <id>
taskd epic create --project <id> <name> [--description <desc>]
taskd epic show <id>
taskd epic close <id>
taskd epic delete <id>

taskd task list --project <id> [--status <s>] [--epic <id>] [--assignee <a>] [--label <l>] [--kind <k>] [--parent <id>]
taskd task create --project <id> <title> [--epic <id>] [--kind <k>] [--parent <id>] [--priority <p>] [--assignee <a>] [--label <l>]...
taskd task show <id>
taskd task update <id> [--title <t>] [--description <d>] [--status <s>] [--priority <p>] [--assignee <a>] [--epic <id>] [--kind <k>]
taskd task done <id>
taskd task output <id> --kind <kind> --ref <ref> [--label <label>]
taskd task outputs <id>
taskd task block <id> --by <dep_id>
taskd task unblock <id> --from <dep_id>
taskd task delete <id>

taskd label list
taskd label create <name> [--color <hex>]
taskd label delete <id>
```

## Testing

- **Unit tests:** `cargo test --workspace` — 21 tests covering all CRUD, cascades, backlog behavior, prefix lookup, ID generation, task outputs, dependencies, and cycle detection
- **E2E tests:** `./tests/cli_e2e.sh` — 35 tests exercising the full CLI against a temp database, covering every command, filters, cascades, sub-tasks, labels, and error cases
