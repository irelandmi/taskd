mod handlers;
mod routes;

use std::sync::{Arc, Mutex};

use clap::Parser;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use taskd_core::db::Database;

#[derive(Parser)]
#[command(name = "taskd-server")]
struct Args {
	#[arg(long, default_value = "3000")]
	port: u16,

	#[arg(long, default_value = "taskd.db")]
	db: String,

	#[arg(long)]
	static_dir: Option<String>,
}

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let args = Args::parse();
	let db = Database::open(&args.db).expect("failed to open database");
	let db = Arc::new(Mutex::new(db));

	let mut app = routes::api_routes()
		.layer(CorsLayer::permissive())
		.with_state(db);

	if let Some(ref dir) = args.static_dir {
		let index = format!("{dir}/index.html");
		app = app.fallback_service(ServeDir::new(dir).fallback(ServeFile::new(index)));
	}

	let addr = format!("0.0.0.0:{}", args.port);
	tracing::info!("listening on {addr}");
	let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
	axum::serve(listener, app).await.unwrap();
}
