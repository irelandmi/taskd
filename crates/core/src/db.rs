use rusqlite::Connection;

use crate::error::Result;

const SCHEMA: &str = "
CREATE TABLE projects (
	id          TEXT PRIMARY KEY,
	name        TEXT NOT NULL,
	description TEXT NOT NULL DEFAULT '',
	created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
	updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE epics (
	id          TEXT PRIMARY KEY,
	project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	name        TEXT NOT NULL,
	description TEXT NOT NULL DEFAULT '',
	status      TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'closed')),
	created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
	updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX idx_epics_project ON epics(project_id);

CREATE TABLE tasks (
	id          TEXT PRIMARY KEY,
	epic_id     TEXT NOT NULL REFERENCES epics(id) ON DELETE CASCADE,
	project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	parent_id   TEXT REFERENCES tasks(id) ON DELETE CASCADE,
	kind        TEXT NOT NULL DEFAULT 'task'
	            CHECK(kind IN ('story', 'task', 'spike', 'bug', 'chore')),
	title       TEXT NOT NULL,
	description TEXT NOT NULL DEFAULT '',
	status      TEXT NOT NULL DEFAULT 'todo'
	            CHECK(status IN ('todo', 'in_progress', 'done', 'cancelled')),
	priority    TEXT NOT NULL DEFAULT 'medium'
	            CHECK(priority IN ('low', 'medium', 'high', 'urgent')),
	assignee    TEXT,
	created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
	updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_epic    ON tasks(epic_id);
CREATE INDEX idx_tasks_status  ON tasks(status);
CREATE INDEX idx_tasks_parent  ON tasks(parent_id);
CREATE INDEX idx_tasks_kind    ON tasks(kind);

CREATE TABLE labels (
	id    TEXT PRIMARY KEY,
	name  TEXT NOT NULL UNIQUE,
	color TEXT NOT NULL DEFAULT '#6b7280'
);

CREATE TABLE task_labels (
	task_id  TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
	label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
	PRIMARY KEY (task_id, label_id)
);
";

pub struct Database {
	pub conn: Connection,
}

impl Database {
	pub fn open(path: &str) -> Result<Self> {
		let conn = Connection::open(path)?;
		let db = Self { conn };
		db.init()?;
		Ok(db)
	}

	pub fn open_in_memory() -> Result<Self> {
		let conn = Connection::open_in_memory()?;
		let db = Self { conn };
		db.init()?;
		Ok(db)
	}

	fn init(&self) -> Result<()> {
		self.conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
		let version: i32 = self.conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
		if version < 1 {
			self.conn.execute_batch(SCHEMA)?;
			self.conn.pragma_update(None, "user_version", 1)?;
		}
		if version < 2 {
			self.conn.execute_batch("
				CREATE TABLE task_events (
					id         TEXT PRIMARY KEY,
					task_id    TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
					kind       TEXT NOT NULL CHECK(kind IN ('comment', 'status_change', 'created', 'updated', 'assigned')),
					message    TEXT NOT NULL DEFAULT '',
					meta       TEXT NOT NULL DEFAULT '{}',
					created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
				);
				CREATE INDEX idx_task_events_task ON task_events(task_id);
			")?;
			self.conn.pragma_update(None, "user_version", 2)?;
		}
		Ok(())
	}
}
