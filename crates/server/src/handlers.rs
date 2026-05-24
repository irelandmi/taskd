use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use taskd_core::db::Database;
use taskd_core::error::Error;
use taskd_core::models::*;

pub type Db = Arc<Mutex<Database>>;

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

// --- Projects ---

pub async fn list_projects(State(db): State<Db>) -> R<Vec<Project>> {
	let db = db.lock().unwrap();
	Ok(Json(db.list_projects()?))
}

pub async fn create_project(State(db): State<Db>, Json(input): Json<CreateProject>) -> R<Project> {
	let db = db.lock().unwrap();
	Ok(Json(db.create_project(input)?))
}

pub async fn get_project(State(db): State<Db>, Path(id): Path<String>) -> R<Project> {
	let db = db.lock().unwrap();
	Ok(Json(db.get_project(&id)?))
}

pub async fn update_project(
	State(db): State<Db>,
	Path(id): Path<String>,
	Json(input): Json<UpdateProject>,
) -> R<Project> {
	let db = db.lock().unwrap();
	Ok(Json(db.update_project(&id, input)?))
}

pub async fn delete_project(
	State(db): State<Db>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = db.lock().unwrap();
	db.delete_project(&id)?;
	Ok(StatusCode::NO_CONTENT)
}

// --- Epics ---

pub async fn list_epics(State(db): State<Db>, Path(pid): Path<String>) -> R<Vec<Epic>> {
	let db = db.lock().unwrap();
	Ok(Json(db.list_epics(&pid)?))
}

pub async fn create_epic(
	State(db): State<Db>,
	Path(pid): Path<String>,
	Json(input): Json<CreateEpic>,
) -> R<Epic> {
	let db = db.lock().unwrap();
	Ok(Json(db.create_epic(&pid, input)?))
}

pub async fn get_epic(State(db): State<Db>, Path(id): Path<String>) -> R<Epic> {
	let db = db.lock().unwrap();
	Ok(Json(db.get_epic(&id)?))
}

pub async fn update_epic(
	State(db): State<Db>,
	Path(id): Path<String>,
	Json(input): Json<UpdateEpic>,
) -> R<Epic> {
	let db = db.lock().unwrap();
	Ok(Json(db.update_epic(&id, input)?))
}

pub async fn delete_epic(
	State(db): State<Db>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = db.lock().unwrap();
	db.delete_epic(&id)?;
	Ok(StatusCode::NO_CONTENT)
}

// --- Tasks ---

pub async fn list_tasks(
	State(db): State<Db>,
	Path(pid): Path<String>,
	Query(filter): Query<TaskFilter>,
) -> R<Vec<Task>> {
	let db = db.lock().unwrap();
	Ok(Json(db.list_tasks(&pid, &filter)?))
}

pub async fn create_task(
	State(db): State<Db>,
	Path(pid): Path<String>,
	Json(input): Json<CreateTask>,
) -> R<Task> {
	let db = db.lock().unwrap();
	Ok(Json(db.create_task(&pid, input)?))
}

pub async fn get_task(State(db): State<Db>, Path(id): Path<String>) -> R<Task> {
	let db = db.lock().unwrap();
	Ok(Json(db.get_task(&id)?))
}

pub async fn update_task(
	State(db): State<Db>,
	Path(id): Path<String>,
	Json(input): Json<UpdateTask>,
) -> R<Task> {
	let db = db.lock().unwrap();
	Ok(Json(db.update_task(&id, input)?))
}

pub async fn delete_task(
	State(db): State<Db>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = db.lock().unwrap();
	db.delete_task(&id)?;
	Ok(StatusCode::NO_CONTENT)
}

pub async fn set_task_labels(
	State(db): State<Db>,
	Path(id): Path<String>,
	Json(label_ids): Json<Vec<String>>,
) -> R<Task> {
	let db = db.lock().unwrap();
	Ok(Json(db.set_task_labels(&id, &label_ids)?))
}

// --- Labels ---

pub async fn list_labels(State(db): State<Db>) -> R<Vec<Label>> {
	let db = db.lock().unwrap();
	Ok(Json(db.list_labels()?))
}

pub async fn create_label(State(db): State<Db>, Json(input): Json<CreateLabel>) -> R<Label> {
	let db = db.lock().unwrap();
	Ok(Json(db.create_label(input)?))
}

pub async fn delete_label(
	State(db): State<Db>,
	Path(id): Path<String>,
) -> std::result::Result<StatusCode, AppError> {
	let db = db.lock().unwrap();
	db.delete_label(&id)?;
	Ok(StatusCode::NO_CONTENT)
}
