use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
	pub id: String,
	pub name: String,
	pub description: String,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
	pub id: String,
	pub project_id: String,
	pub name: String,
	pub description: String,
	pub status: String,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
	pub id: String,
	pub project_id: String,
	pub epic_id: String,
	pub parent_id: Option<String>,
	pub kind: String,
	pub title: String,
	pub description: String,
	pub status: String,
	pub priority: String,
	pub assignee: Option<String>,
	pub labels: Vec<Label>,
	#[serde(default)]
	pub children: Vec<Task>,
	pub created_at: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
	pub id: String,
	pub name: String,
	pub color: String,
}

// Input types for creation/updates

#[derive(Debug, Deserialize)]
pub struct CreateProject {
	pub name: String,
	#[serde(default)]
	pub description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateProject {
	pub name: Option<String>,
	pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEpic {
	pub name: String,
	#[serde(default)]
	pub description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateEpic {
	pub name: Option<String>,
	pub description: Option<String>,
	pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTask {
	pub title: String,
	#[serde(default)]
	pub description: String,
	#[serde(default)]
	pub epic_id: Option<String>,
	pub parent_id: Option<String>,
	#[serde(default = "default_kind")]
	pub kind: String,
	#[serde(default = "default_priority")]
	pub priority: String,
	pub assignee: Option<String>,
	#[serde(default)]
	pub labels: Vec<String>,
}

fn default_kind() -> String {
	"task".to_string()
}

fn default_priority() -> String {
	"medium".to_string()
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateTask {
	pub title: Option<String>,
	pub description: Option<String>,
	pub epic_id: Option<String>,
	pub status: Option<String>,
	pub priority: Option<String>,
	pub assignee: Option<String>,
	pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabel {
	pub name: String,
	#[serde(default = "default_color")]
	pub color: String,
}

fn default_color() -> String {
	"#6b7280".to_string()
}

#[derive(Debug, Deserialize, Default)]
pub struct TaskFilter {
	pub status: Option<String>,
	pub epic_id: Option<String>,
	pub assignee: Option<String>,
	pub label: Option<String>,
	pub kind: Option<String>,
	pub parent_id: Option<String>,
}
