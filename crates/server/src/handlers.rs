use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use taskd_core::db::Database;
use taskd_core::error::Error;
use taskd_core::models::*;

pub type Db = Arc<Mutex<Database>>;
pub type Tx = broadcast::Sender<SseEvent>;

#[derive(Clone)]
pub struct AppState {
	pub db: Db,
	pub tx: Tx,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct SseEvent {
	#[serde(rename = "type")]
	pub kind: String,
	pub project_id: Option<String>,
	pub task_id: Option<String>,
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, msg) = match self.0 {
			Error::NotFound(m) => (StatusCode::NOT_FOUND, m),
			Error::Conflict(m) => (StatusCode::CONFLICT, m),
			Error::InvalidInput(m) => (StatusCode::BAD_REQUEST, m),
			Error::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
		};
		(status, Json(serde_json::json!({ "error": msg }))).into_response()
	}
}

pub struct AppError(Error);
impl From<Error> for AppError {
	fn from(e: Error) -> Self {
		Self(e)
	}
}

type R<T> = std::result::Result<Json<T>, AppError>;

fn broadcast(tx: &Tx, kind: &str, project_id: Option<&str>, task_id: Option<&str>) {
	let _ = tx.send(SseEvent {
		kind: kind.to_string(),
		project_id: project_id.map(String::from),
		task_id: task_id.map(String::from),
	});
}

// --- SSE ---

pub async fn events(
	State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
	let rx = state.tx.subscribe();
	let stream = BroadcastStream::new(rx).filter_map(|msg| {
		match msg {
			Ok(evt) => {
				let data = serde_json::to_string(&evt).unwrap_or_default();
				Some(Ok(Event::default().data(data)))
			}
			Err(_) => None,
		}
	});
	Sse::new(stream).keep_alive(KeepAlive::default())
}

// --- Projects ---

pub async fn list_projects(State(state): State<AppState>) -> R<Vec<Project>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_projects()?))
}

pub async fn create_project(State(state): State<AppState>, Json(input): Json<CreateProject>) -> R<Project> {
	let db = state.db.lock().unwrap();
	let p = db.create_project(input)?;
	broadcast(&state.tx, "project_created", Some(&p.id), None);
	Ok(Json(p))
}

pub async fn get_project(State(state): State<AppState>, Path(id): Path<String>) -> R<Project> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.get_project(&id)?))
}

pub async fn update_project(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(input): Json<UpdateProject>,
) -> R<Project> {
	let db = state.db.lock().unwrap();
	let p = db.update_project(&id, input)?;
	broadcast(&state.tx, "project_updated", Some(&p.id), None);
	Ok(Json(p))
}

pub async fn delete_project(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.delete_project(&id)?;
	broadcast(&state.tx, "project_deleted", Some(&id), None);
	Ok(StatusCode::NO_CONTENT)
}

// --- Epics ---

pub async fn list_epics(State(state): State<AppState>, Path(pid): Path<String>) -> R<Vec<Epic>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_epics(&pid)?))
}

pub async fn create_epic(
	State(state): State<AppState>,
	Path(pid): Path<String>,
	Json(input): Json<CreateEpic>,
) -> R<Epic> {
	let db = state.db.lock().unwrap();
	let e = db.create_epic(&pid, input)?;
	broadcast(&state.tx, "epic_created", Some(&e.project_id), None);
	Ok(Json(e))
}

pub async fn get_epic(State(state): State<AppState>, Path(id): Path<String>) -> R<Epic> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.get_epic(&id)?))
}

pub async fn update_epic(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(input): Json<UpdateEpic>,
) -> R<Epic> {
	let db = state.db.lock().unwrap();
	let e = db.update_epic(&id, input)?;
	broadcast(&state.tx, "epic_updated", Some(&e.project_id), None);
	Ok(Json(e))
}

pub async fn delete_epic(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.delete_epic(&id)?;
	broadcast(&state.tx, "epic_deleted", None, None);
	Ok(StatusCode::NO_CONTENT)
}

// --- Tasks ---

pub async fn list_tasks(
	State(state): State<AppState>,
	Path(pid): Path<String>,
	Query(filter): Query<TaskFilter>,
) -> R<Vec<Task>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_tasks(&pid, &filter)?))
}

pub async fn create_task(
	State(state): State<AppState>,
	Path(pid): Path<String>,
	Json(input): Json<CreateTask>,
) -> R<Task> {
	let db = state.db.lock().unwrap();
	let t = db.create_task(&pid, input)?;
	broadcast(&state.tx, "task_created", Some(&t.project_id), Some(&t.id));
	Ok(Json(t))
}

pub async fn get_task(State(state): State<AppState>, Path(id): Path<String>) -> R<Task> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.get_task(&id)?))
}

pub async fn update_task(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(input): Json<UpdateTask>,
) -> R<Task> {
	let db = state.db.lock().unwrap();
	let t = db.update_task(&id, input)?;
	broadcast(&state.tx, "task_updated", Some(&t.project_id), Some(&t.id));
	Ok(Json(t))
}

pub async fn delete_task(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.delete_task(&id)?;
	broadcast(&state.tx, "task_deleted", None, Some(&id));
	Ok(StatusCode::NO_CONTENT)
}

pub async fn set_task_labels(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(label_ids): Json<Vec<String>>,
) -> R<Task> {
	let db = state.db.lock().unwrap();
	let t = db.set_task_labels(&id, &label_ids)?;
	broadcast(&state.tx, "task_updated", Some(&t.project_id), Some(&t.id));
	Ok(Json(t))
}

// --- Task Events ---

pub async fn list_task_events(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> R<Vec<TaskEvent>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_task_events(&id)?))
}

pub async fn create_task_event(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(input): Json<CreateTaskEvent>,
) -> R<TaskEvent> {
	let db = state.db.lock().unwrap();
	let task = db.get_task(&id)?;
	let evt = db.log_event(&task.id, "comment", &input.message, "{}")?;
	broadcast(&state.tx, "task_event", Some(&task.project_id), Some(&task.id));
	Ok(Json(evt))
}

// --- Task Outputs ---

pub async fn list_task_outputs(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> R<Vec<TaskOutput>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_task_outputs(&id)?))
}

pub async fn create_task_output(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(input): Json<CreateTaskOutput>,
) -> R<TaskOutput> {
	let db = state.db.lock().unwrap();
	let output = db.add_task_output(&id, input)?;
	broadcast(&state.tx, "task_updated", None, Some(&id));
	Ok(Json(output))
}

// --- Task Dependencies ---

pub async fn add_dependency(
	State(state): State<AppState>,
	Path(id): Path<String>,
	Json(body): Json<AddDependencyBody>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.add_dependency(&id, &body.depends_on)?;
	broadcast(&state.tx, "task_updated", None, Some(&id));
	Ok(StatusCode::CREATED)
}

pub async fn remove_dependency(
	State(state): State<AppState>,
	Path((id, dep_id)): Path<(String, String)>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.remove_dependency(&id, &dep_id)?;
	broadcast(&state.tx, "task_updated", None, Some(&id));
	Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize)]
pub struct AddDependencyBody {
	pub depends_on: String,
}

// --- Labels ---

pub async fn list_labels(State(state): State<AppState>) -> R<Vec<Label>> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.list_labels()?))
}

pub async fn create_label(State(state): State<AppState>, Json(input): Json<CreateLabel>) -> R<Label> {
	let db = state.db.lock().unwrap();
	Ok(Json(db.create_label(input)?))
}

pub async fn delete_label(
	State(state): State<AppState>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = state.db.lock().unwrap();
	db.delete_label(&id)?;
	Ok(StatusCode::NO_CONTENT)
}
