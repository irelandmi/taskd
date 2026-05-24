use axum::routing::{delete, get, put};
use axum::Router;

use crate::handlers::{self, Db};

pub fn api_routes() -> Router<Db> {
	Router::new()
		// Projects
		.route("/api/projects", get(handlers::list_projects).post(handlers::create_project))
		.route(
			"/api/projects/{id}",
			get(handlers::get_project)
				.patch(handlers::update_project)
				.delete(handlers::delete_project),
		)
		// Epics
		.route(
			"/api/projects/{pid}/epics",
			get(handlers::list_epics).post(handlers::create_epic),
		)
		.route(
			"/api/epics/{id}",
			get(handlers::get_epic)
				.patch(handlers::update_epic)
				.delete(handlers::delete_epic),
		)
		// Tasks
		.route(
			"/api/projects/{pid}/tasks",
			get(handlers::list_tasks).post(handlers::create_task),
		)
		.route(
			"/api/tasks/{id}",
			get(handlers::get_task)
				.patch(handlers::update_task)
				.delete(handlers::delete_task),
		)
		.route("/api/tasks/{id}/labels", put(handlers::set_task_labels))
		// Labels
		.route("/api/labels", get(handlers::list_labels).post(handlers::create_label))
		.route("/api/labels/{id}", delete(handlers::delete_label))
}
