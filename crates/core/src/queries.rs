use rusqlite::params;

use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::*;
use crate::names::generate_name;

fn new_id() -> String {
	generate_name()
}

fn now() -> String {
	let dt = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.unwrap()
		.as_secs();
	let secs = dt % 60;
	let mins = (dt / 60) % 60;
	let hours = (dt / 3600) % 24;
	let days = dt / 86400;
	// Good enough ISO 8601 — matches SQLite's strftime output
	let (y, m, d) = days_to_ymd(days);
	format!("{y:04}-{m:02}-{d:02}T{hours:02}:{mins:02}:{secs:02}Z")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
	// Algorithm from http://howardhinnant.github.io/date_algorithms.html
	let z = days + 719468;
	let era = z / 146097;
	let doe = z - era * 146097;
	let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
	let y = yoe + era * 400;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let d = doy - (153 * mp + 2) / 5 + 1;
	let m = if mp < 10 { mp + 3 } else { mp - 9 };
	let y = if m <= 2 { y + 1 } else { y };
	(y, m, d)
}

const TASK_COLUMNS: &str = "id, project_id, epic_id, parent_id, kind, title, description, status, priority, assignee, created_at, updated_at";

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
	Ok(Task {
		id: row.get(0)?,
		project_id: row.get(1)?,
		epic_id: row.get(2)?,
		parent_id: row.get(3)?,
		kind: row.get(4)?,
		title: row.get(5)?,
		description: row.get(6)?,
		status: row.get(7)?,
		priority: row.get(8)?,
		assignee: row.get(9)?,
		labels: vec![],
		children: vec![],
		created_at: row.get(10)?,
		updated_at: row.get(11)?,
	})
}

fn lookup_id(conn: &rusqlite::Connection, table: &str, id: &str) -> Result<String> {
	let sql = format!("SELECT id FROM {table} WHERE id = ?1 OR id LIKE ?2 LIMIT 2");
	let mut stmt = conn.prepare(&sql)?;
	let mut rows = stmt.query_map(params![id, format!("{id}%")], |row| row.get::<_, String>(0))?;
	let first = rows.next()
		.ok_or_else(|| Error::NotFound(format!("{table} {id}")))?
		.map_err(|_| Error::NotFound(format!("{table} {id}")))?;
	if rows.next().is_some() {
		return Err(Error::Conflict(format!("ambiguous prefix '{id}' matches multiple {table}")));
	}
	Ok(first)
}

impl Database {
	// --- Projects ---

	pub fn list_projects(&self) -> Result<Vec<Project>> {
		let mut stmt = self.conn.prepare(
			"SELECT id, name, description, created_at, updated_at FROM projects ORDER BY created_at",
		)?;
		let rows = stmt.query_map([], |row| {
			Ok(Project {
				id: row.get(0)?,
				name: row.get(1)?,
				description: row.get(2)?,
				created_at: row.get(3)?,
				updated_at: row.get(4)?,
			})
		})?;
		rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
	}

	pub fn create_project(&self, input: CreateProject) -> Result<Project> {
		let id = new_id();
		let ts = now();
		self.conn.execute_batch("BEGIN")?;
		let result = (|| -> Result<Project> {
			self.conn.execute(
				"INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
				params![id, input.name, input.description, ts, ts],
			)?;
			let epic_id = new_id();
			let epic_ts = now();
			self.conn.execute(
				"INSERT INTO epics (id, project_id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
				params![epic_id, id, "Backlog", "", epic_ts, epic_ts],
			)?;
			self.conn.execute_batch("COMMIT")?;
			self.get_project(&id)
		})();
		if result.is_err() {
			let _ = self.conn.execute_batch("ROLLBACK");
		}
		result
	}

	pub fn get_backlog_epic(&self, project_id: &str) -> Result<Epic> {
		let project = self.get_project(project_id)?;
		match self.conn.query_row(
			"SELECT id, project_id, name, description, status, created_at, updated_at FROM epics WHERE project_id = ?1 AND name = 'Backlog'",
			params![project.id],
			|row| {
				Ok(Epic {
					id: row.get(0)?,
					project_id: row.get(1)?,
					name: row.get(2)?,
					description: row.get(3)?,
					status: row.get(4)?,
					created_at: row.get(5)?,
					updated_at: row.get(6)?,
				})
			},
		) {
			Ok(epic) => Ok(epic),
			Err(_) => self.create_epic(&project.id, CreateEpic { name: "Backlog".into(), description: String::new() }),
		}
	}

	pub fn get_project(&self, id: &str) -> Result<Project> {
		let resolved = lookup_id(&self.conn, "projects", id)?;
		self.conn
			.query_row(
				"SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1",
				params![resolved],
				|row| {
					Ok(Project {
						id: row.get(0)?,
						name: row.get(1)?,
						description: row.get(2)?,
						created_at: row.get(3)?,
						updated_at: row.get(4)?,
					})
				},
			)
			.map_err(|_| Error::NotFound(format!("project {id}")))
	}

	pub fn update_project(&self, id: &str, input: UpdateProject) -> Result<Project> {
		let project = self.get_project(id)?;
		let name = input.name.unwrap_or(project.name);
		let description = input.description.unwrap_or(project.description);
		let ts = now();
		self.conn.execute(
			"UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
			params![name, description, ts, project.id],
		)?;
		self.get_project(&project.id)
	}

	pub fn delete_project(&self, id: &str) -> Result<()> {
		let project = self.get_project(id)?;
		self.conn.execute("DELETE FROM projects WHERE id = ?1", params![project.id])?;
		Ok(())
	}

	// --- Epics ---

	pub fn list_epics(&self, project_id: &str) -> Result<Vec<Epic>> {
		let project = self.get_project(project_id)?;
		let mut stmt = self.conn.prepare(
			"SELECT id, project_id, name, description, status, created_at, updated_at FROM epics WHERE project_id = ?1 ORDER BY created_at",
		)?;
		let rows = stmt.query_map(params![project.id], |row| {
			Ok(Epic {
				id: row.get(0)?,
				project_id: row.get(1)?,
				name: row.get(2)?,
				description: row.get(3)?,
				status: row.get(4)?,
				created_at: row.get(5)?,
				updated_at: row.get(6)?,
			})
		})?;
		rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
	}

	pub fn create_epic(&self, project_id: &str, input: CreateEpic) -> Result<Epic> {
		let project = self.get_project(project_id)?;
		let id = new_id();
		let ts = now();
		self.conn.execute(
			"INSERT INTO epics (id, project_id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			params![id, project.id, input.name, input.description, ts, ts],
		)?;
		self.get_epic(&id)
	}

	pub fn get_epic(&self, id: &str) -> Result<Epic> {
		let resolved = lookup_id(&self.conn, "epics", id)?;
		self.conn
			.query_row(
				"SELECT id, project_id, name, description, status, created_at, updated_at FROM epics WHERE id = ?1",
				params![resolved],
				|row| {
					Ok(Epic {
						id: row.get(0)?,
						project_id: row.get(1)?,
						name: row.get(2)?,
						description: row.get(3)?,
						status: row.get(4)?,
						created_at: row.get(5)?,
						updated_at: row.get(6)?,
					})
				},
			)
			.map_err(|_| Error::NotFound(format!("epic {id}")))
	}

	pub fn update_epic(&self, id: &str, input: UpdateEpic) -> Result<Epic> {
		let epic = self.get_epic(id)?;
		let name = input.name.unwrap_or(epic.name);
		let description = input.description.unwrap_or(epic.description);
		let status = input.status.unwrap_or(epic.status);
		let ts = now();
		self.conn.execute(
			"UPDATE epics SET name = ?1, description = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
			params![name, description, status, ts, epic.id],
		)?;
		self.get_epic(&epic.id)
	}

	pub fn delete_epic(&self, id: &str) -> Result<()> {
		let epic = self.get_epic(id)?;
		self.conn.execute("DELETE FROM epics WHERE id = ?1", params![epic.id])?;
		Ok(())
	}

	// --- Tasks ---

	pub fn list_tasks(&self, project_id: &str, filter: &TaskFilter) -> Result<Vec<Task>> {
		let project = self.get_project(project_id)?;
		let mut sql = format!(
			"SELECT {TASK_COLUMNS} FROM tasks WHERE project_id = ?1",
		);
		let mut param_values: Vec<String> = vec![project.id.clone()];

		if let Some(ref status) = filter.status {
			param_values.push(status.clone());
			sql.push_str(&format!(" AND status = ?{}", param_values.len()));
		}
		if let Some(ref epic_id) = filter.epic_id {
			param_values.push(epic_id.clone());
			sql.push_str(&format!(" AND epic_id = ?{}", param_values.len()));
		}
		if let Some(ref assignee) = filter.assignee {
			param_values.push(assignee.clone());
			sql.push_str(&format!(" AND assignee = ?{}", param_values.len()));
		}
		if let Some(ref label) = filter.label {
			param_values.push(label.clone());
			sql.push_str(&format!(
				" AND id IN (SELECT task_id FROM task_labels JOIN labels ON labels.id = task_labels.label_id WHERE labels.name = ?{})",
				param_values.len()
			));
		}
		if let Some(ref kind) = filter.kind {
			param_values.push(kind.clone());
			sql.push_str(&format!(" AND kind = ?{}", param_values.len()));
		}
		if let Some(ref parent_id) = filter.parent_id {
			param_values.push(parent_id.clone());
			sql.push_str(&format!(" AND parent_id = ?{}", param_values.len()));
		}
		sql.push_str(" ORDER BY created_at");

		let mut stmt = self.conn.prepare(&sql)?;
		let param_refs: Vec<&dyn rusqlite::types::ToSql> =
			param_values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
		let rows = stmt.query_map(param_refs.as_slice(), row_to_task)?;
		let mut tasks: Vec<Task> =
			rows.collect::<std::result::Result<Vec<_>, _>>()?;
		for task in &mut tasks {
			task.labels = self.get_task_labels(&task.id)?;
		}
		Ok(tasks)
	}

	pub fn create_task(&self, project_id: &str, input: CreateTask) -> Result<Task> {
		let project = self.get_project(project_id)?;
		let epic_id = match input.epic_id {
			Some(ref eid) => { self.get_epic(eid)?; eid.clone() }
			None => self.get_backlog_epic(&project.id)?.id,
		};
		if let Some(ref parent_id) = input.parent_id {
			self.get_task(parent_id)?;
		}
		let id = new_id();
		let ts = now();
		self.conn.execute_batch("BEGIN")?;
		let result = (|| -> Result<()> {
			self.conn.execute(
				"INSERT INTO tasks (id, project_id, epic_id, parent_id, kind, title, description, priority, assignee, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
				params![id, project.id, epic_id, input.parent_id, input.kind, input.title, input.description, input.priority, input.assignee, ts, ts],
			)?;
			for label_name in &input.labels {
				if let Ok(label) = self.get_label_by_name(label_name) {
					self.conn.execute(
						"INSERT OR IGNORE INTO task_labels (task_id, label_id) VALUES (?1, ?2)",
						params![id, label.id],
					)?;
				}
			}
			self.conn.execute_batch("COMMIT")?;
			Ok(())
		})();
		if result.is_err() {
			let _ = self.conn.execute_batch("ROLLBACK");
		}
		result?;
		let task = self.get_task(&id)?;
		self.log_event(&task.id, "created", &format!("created {} '{}'", task.kind, task.title), "{}")?;
		Ok(task)
	}

	pub fn get_task(&self, id: &str) -> Result<Task> {
		let resolved = lookup_id(&self.conn, "tasks", id)?;
		let sql = format!(
			"SELECT {TASK_COLUMNS} FROM tasks WHERE id = ?1"
		);
		let mut task = self.conn
			.query_row(&sql, params![resolved], row_to_task)
			.map_err(|_| Error::NotFound(format!("task {id}")))?;
		task.labels = self.get_task_labels(&task.id)?;
		// Load one level of children
		let child_sql = format!(
			"SELECT {TASK_COLUMNS} FROM tasks WHERE parent_id = ?1 ORDER BY created_at"
		);
		let mut stmt = self.conn.prepare(&child_sql)?;
		let rows = stmt.query_map(params![task.id], row_to_task)?;
		let mut children: Vec<Task> = rows.collect::<std::result::Result<Vec<_>, _>>()?;
		for child in &mut children {
			child.labels = self.get_task_labels(&child.id)?;
		}
		task.children = children;
		Ok(task)
	}

	pub fn update_task(&self, id: &str, input: UpdateTask) -> Result<Task> {
		let task = self.get_task(id)?;
		let old_status = task.status.clone();
		let old_assignee = task.assignee.clone();
		let title = input.title.unwrap_or(task.title);
		let description = input.description.unwrap_or(task.description);
		let epic_id = input.epic_id.unwrap_or(task.epic_id);
		let status = input.status.unwrap_or(old_status.clone());
		let priority = input.priority.unwrap_or(task.priority);
		let kind = input.kind.unwrap_or(task.kind);
		let assignee = if input.assignee.is_some() { input.assignee } else { old_assignee.clone() };
		let ts = now();
		self.conn.execute(
			"UPDATE tasks SET title = ?1, description = ?2, epic_id = ?3, status = ?4, priority = ?5, assignee = ?6, kind = ?7, updated_at = ?8 WHERE id = ?9",
			params![title, description, epic_id, status, priority, assignee, kind, ts, task.id],
		)?;
		let updated = self.get_task(&task.id)?;
		if updated.status != old_status {
			self.log_event(&task.id, "status_change",
				&format!("{} → {}", old_status, updated.status),
				&format!("{{\"from\":\"{old_status}\",\"to\":\"{}\"}}", updated.status),
			)?;
		}
		if updated.assignee != old_assignee {
			let msg = match (&old_assignee, &updated.assignee) {
				(None, Some(a)) => format!("assigned to {a}"),
				(Some(_), None) => "unassigned".to_string(),
				(Some(a), Some(b)) => format!("reassigned {a} → {b}"),
				_ => String::new(),
			};
			if !msg.is_empty() {
				self.log_event(&task.id, "assigned", &msg, "{}")?;
			}
		}
		Ok(updated)
	}

	pub fn delete_task(&self, id: &str) -> Result<()> {
		let task = self.get_task(id)?;
		self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![task.id])?;
		Ok(())
	}

	pub fn set_task_labels(&self, task_id: &str, label_ids: &[String]) -> Result<Task> {
		let task = self.get_task(task_id)?;
		self.conn.execute_batch("BEGIN")?;
		let result = (|| -> Result<()> {
			self.conn.execute("DELETE FROM task_labels WHERE task_id = ?1", params![task.id])?;
			for label_id in label_ids {
				self.conn.execute(
					"INSERT INTO task_labels (task_id, label_id) VALUES (?1, ?2)",
					params![task.id, label_id],
				)?;
			}
			self.conn.execute_batch("COMMIT")?;
			Ok(())
		})();
		if result.is_err() {
			let _ = self.conn.execute_batch("ROLLBACK");
		}
		result?;
		self.get_task(&task.id)
	}

	// --- Labels ---

	pub fn list_labels(&self) -> Result<Vec<Label>> {
		let mut stmt = self.conn.prepare("SELECT id, name, color FROM labels ORDER BY name")?;
		let rows = stmt.query_map([], |row| {
			Ok(Label {
				id: row.get(0)?,
				name: row.get(1)?,
				color: row.get(2)?,
			})
		})?;
		rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
	}

	pub fn create_label(&self, input: CreateLabel) -> Result<Label> {
		let id = new_id();
		self.conn.execute(
			"INSERT INTO labels (id, name, color) VALUES (?1, ?2, ?3)",
			params![id, input.name, input.color],
		)?;
		self.get_label(&id)
	}

	pub fn get_label(&self, id: &str) -> Result<Label> {
		let resolved = lookup_id(&self.conn, "labels", id)?;
		self.conn
			.query_row(
				"SELECT id, name, color FROM labels WHERE id = ?1",
				params![resolved],
				|row| {
					Ok(Label {
						id: row.get(0)?,
						name: row.get(1)?,
						color: row.get(2)?,
					})
				},
			)
			.map_err(|_| Error::NotFound(format!("label {id}")))
	}

	pub fn get_label_by_name(&self, name: &str) -> Result<Label> {
		self.conn
			.query_row(
				"SELECT id, name, color FROM labels WHERE name = ?1",
				params![name],
				|row| {
					Ok(Label {
						id: row.get(0)?,
						name: row.get(1)?,
						color: row.get(2)?,
					})
				},
			)
			.map_err(|_| Error::NotFound(format!("label {name}")))
	}

	pub fn delete_label(&self, id: &str) -> Result<()> {
		let label = self.get_label(id)?;
		self.conn.execute("DELETE FROM labels WHERE id = ?1", params![label.id])?;
		Ok(())
	}

	// --- Task Events ---

	pub fn log_event(&self, task_id: &str, kind: &str, message: &str, meta: &str) -> Result<TaskEvent> {
		let task = self.get_task(task_id)?;
		let id = new_id();
		let ts = now();
		self.conn.execute(
			"INSERT INTO task_events (id, task_id, kind, message, meta, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			params![id, task.id, kind, message, meta, ts],
		)?;
		self.conn
			.query_row(
				"SELECT id, task_id, kind, message, meta, created_at FROM task_events WHERE id = ?1",
				params![id],
				|row| Ok(TaskEvent {
					id: row.get(0)?,
					task_id: row.get(1)?,
					kind: row.get(2)?,
					message: row.get(3)?,
					meta: row.get(4)?,
					created_at: row.get(5)?,
				}),
			)
			.map_err(|_| Error::NotFound(format!("task_event {id}")))
	}

	pub fn list_task_events(&self, task_id: &str) -> Result<Vec<TaskEvent>> {
		let task = self.get_task(task_id)?;
		let mut stmt = self.conn.prepare(
			"SELECT id, task_id, kind, message, meta, created_at FROM task_events WHERE task_id = ?1 ORDER BY created_at",
		)?;
		let rows = stmt.query_map(params![task.id], |row| {
			Ok(TaskEvent {
				id: row.get(0)?,
				task_id: row.get(1)?,
				kind: row.get(2)?,
				message: row.get(3)?,
				meta: row.get(4)?,
				created_at: row.get(5)?,
			})
		})?;
		rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
	}

	fn get_task_labels(&self, task_id: &str) -> Result<Vec<Label>> {
		let mut stmt = self.conn.prepare(
			"SELECT l.id, l.name, l.color FROM labels l JOIN task_labels tl ON l.id = tl.label_id WHERE tl.task_id = ?1",
		)?;
		let rows = stmt.query_map(params![task_id], |row| {
			Ok(Label {
				id: row.get(0)?,
				name: row.get(1)?,
				color: row.get(2)?,
			})
		})?;
		rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_db() -> Database {
		Database::open_in_memory().unwrap()
	}

	fn create_project_and_epic(db: &Database) -> (Project, Epic) {
		let p = db.create_project(CreateProject { name: "P".into(), description: "".into() }).unwrap();
		let e = db.create_epic(&p.id, CreateEpic { name: "E".into(), description: "".into() }).unwrap();
		(p, e)
	}

	#[test]
	fn test_project_crud() {
		let db = test_db();
		let p = db.create_project(CreateProject { name: "Test".into(), description: "".into() }).unwrap();
		assert_eq!(p.name, "Test");

		let projects = db.list_projects().unwrap();
		assert_eq!(projects.len(), 1);

		let p = db.update_project(&p.id, UpdateProject { name: Some("Updated".into()), ..Default::default() }).unwrap();
		assert_eq!(p.name, "Updated");

		// Prefix lookup
		let p2 = db.get_project(&p.id[..8]).unwrap();
		assert_eq!(p2.id, p.id);

		db.delete_project(&p.id).unwrap();
		assert!(db.list_projects().unwrap().is_empty());
	}

	#[test]
	fn test_epic_crud() {
		let db = test_db();
		let p = db.create_project(CreateProject { name: "P".into(), description: "".into() }).unwrap();
		let e = db.create_epic(&p.id, CreateEpic { name: "Epic 1".into(), description: "".into() }).unwrap();
		assert_eq!(e.status, "open");

		let e = db.update_epic(&e.id, UpdateEpic { status: Some("closed".into()), ..Default::default() }).unwrap();
		assert_eq!(e.status, "closed");

		let epics = db.list_epics(&p.id).unwrap();
		assert_eq!(epics.len(), 2); // Backlog + Epic 1
	}

	#[test]
	fn test_task_crud() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let t = db.create_task(&p.id, CreateTask {
			title: "Do thing".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "task".into(),
			priority: "high".into(),
			assignee: Some("alice".into()),
			labels: vec![],
		}).unwrap();
		assert_eq!(t.status, "todo");
		assert_eq!(t.priority, "high");
		assert_eq!(t.kind, "task");
		assert_eq!(t.epic_id, e.id);

		let t = db.update_task(&t.id, UpdateTask { status: Some("done".into()), ..Default::default() }).unwrap();
		assert_eq!(t.status, "done");

		let tasks = db.list_tasks(&p.id, &TaskFilter::default()).unwrap();
		assert_eq!(tasks.len(), 1);
	}

	#[test]
	fn test_task_kinds() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		for kind in &["story", "task", "spike", "bug", "chore"] {
			let t = db.create_task(&p.id, CreateTask {
				title: format!("A {kind}"),
				description: "".into(),
				epic_id: Some(e.id.clone()),
				parent_id: None,
				kind: kind.to_string(),
				priority: "medium".into(),
				assignee: None,
				labels: vec![],
			}).unwrap();
			assert_eq!(t.kind, *kind);
		}
		let stories = db.list_tasks(&p.id, &TaskFilter { kind: Some("story".into()), ..Default::default() }).unwrap();
		assert_eq!(stories.len(), 1);
		assert_eq!(stories[0].kind, "story");
	}

	#[test]
	fn test_parent_child() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let parent = db.create_task(&p.id, CreateTask {
			title: "Parent story".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "story".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();

		let child = db.create_task(&p.id, CreateTask {
			title: "Sub-task".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: Some(parent.id.clone()),
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		assert_eq!(child.parent_id.as_deref(), Some(parent.id.as_str()));

		let fetched = db.get_task(&parent.id).unwrap();
		assert_eq!(fetched.children.len(), 1);
		assert_eq!(fetched.children[0].id, child.id);
	}

	#[test]
	fn test_parent_cascade_delete() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let parent = db.create_task(&p.id, CreateTask {
			title: "Parent".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "story".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		db.create_task(&p.id, CreateTask {
			title: "Child".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: Some(parent.id.clone()),
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();

		db.delete_task(&parent.id).unwrap();
		let tasks = db.list_tasks(&p.id, &TaskFilter::default()).unwrap();
		assert!(tasks.is_empty());
	}

	#[test]
	fn test_kind_filter() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		db.create_task(&p.id, CreateTask {
			title: "A bug".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "bug".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		db.create_task(&p.id, CreateTask {
			title: "A story".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "story".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();

		let bugs = db.list_tasks(&p.id, &TaskFilter { kind: Some("bug".into()), ..Default::default() }).unwrap();
		assert_eq!(bugs.len(), 1);
		assert_eq!(bugs[0].title, "A bug");
	}

	#[test]
	fn test_update_kind() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let t = db.create_task(&p.id, CreateTask {
			title: "Something".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		let t = db.update_task(&t.id, UpdateTask { kind: Some("bug".into()), ..Default::default() }).unwrap();
		assert_eq!(t.kind, "bug");
	}

	#[test]
	fn test_labels() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let label = db.create_label(CreateLabel { name: "bug".into(), color: "#ff0000".into() }).unwrap();

		let t = db.create_task(&p.id, CreateTask {
			title: "Fix bug".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec!["bug".into()],
		}).unwrap();
		assert_eq!(t.labels.len(), 1);
		assert_eq!(t.labels[0].name, "bug");

		let t = db.set_task_labels(&t.id, &[]).unwrap();
		assert!(t.labels.is_empty());

		let t = db.set_task_labels(&t.id, &[label.id]).unwrap();
		assert_eq!(t.labels.len(), 1);
	}

	#[test]
	fn test_epic_cascade_deletes_tasks() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		db.create_task(&p.id, CreateTask {
			title: "Task in epic".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		db.delete_epic(&e.id).unwrap();
		let tasks = db.list_tasks(&p.id, &TaskFilter::default()).unwrap();
		assert!(tasks.is_empty());
	}

	#[test]
	fn test_project_creates_backlog_epic() {
		let db = test_db();
		let p = db.create_project(CreateProject { name: "P".into(), description: "".into() }).unwrap();
		let epics = db.list_epics(&p.id).unwrap();
		assert_eq!(epics.len(), 1);
		assert_eq!(epics[0].name, "Backlog");
	}

	#[test]
	fn test_task_without_epic_uses_backlog() {
		let db = test_db();
		let p = db.create_project(CreateProject { name: "P".into(), description: "".into() }).unwrap();
		let t = db.create_task(&p.id, CreateTask {
			title: "Quick fix".into(),
			description: "".into(),
			epic_id: None,
			parent_id: None,
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		let backlog = db.get_backlog_epic(&p.id).unwrap();
		assert_eq!(t.epic_id, backlog.id);
	}

	#[test]
	fn test_task_with_explicit_epic() {
		let db = test_db();
		let (p, e) = create_project_and_epic(&db);
		let t = db.create_task(&p.id, CreateTask {
			title: "Planned".into(),
			description: "".into(),
			epic_id: Some(e.id.clone()),
			parent_id: None,
			kind: "task".into(),
			priority: "medium".into(),
			assignee: None,
			labels: vec![],
		}).unwrap();
		assert_eq!(t.epic_id, e.id);
	}

	#[test]
	fn test_ambiguous_prefix_returns_error() {
		let db = test_db();
		db.create_project(CreateProject { name: "A".into(), description: "".into() }).unwrap();
		db.create_project(CreateProject { name: "B".into(), description: "".into() }).unwrap();
		// Single-char prefix likely matches both — should get Conflict error
		// Use an empty string to guarantee ambiguity
		let result = db.get_project("");
		assert!(result.is_err());
	}

	#[test]
	fn test_exact_id_lookup_works() {
		let db = test_db();
		let p = db.create_project(CreateProject { name: "Test".into(), description: "".into() }).unwrap();
		let fetched = db.get_project(&p.id).unwrap();
		assert_eq!(fetched.id, p.id);
	}
}
