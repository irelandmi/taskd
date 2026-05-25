use clap::{Parser, Subcommand};
use tabled::{Table, Tabled};

use taskd_core::db::Database;
use taskd_core::models::*;

#[derive(Parser)]
#[command(name = "taskd", about = "Lightweight project management")]
struct Cli {
	#[arg(long, default_value = "taskd.db")]
	db: String,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	Project {
		#[command(subcommand)]
		cmd: ProjectCmd,
	},
	Epic {
		#[command(subcommand)]
		cmd: EpicCmd,
	},
	Task {
		#[command(subcommand)]
		cmd: TaskCmd,
	},
	Label {
		#[command(subcommand)]
		cmd: LabelCmd,
	},
	Serve {
		#[arg(long, default_value = "3000")]
		port: u16,
		#[arg(long)]
		static_dir: Option<String>,
	},
}

#[derive(Subcommand)]
enum ProjectCmd {
	List,
	Create {
		name: String,
		#[arg(long, default_value = "")]
		description: String,
	},
	Show {
		id: String,
	},
	Delete {
		id: String,
	},
}

#[derive(Subcommand)]
enum EpicCmd {
	List {
		#[arg(long)]
		project: String,
	},
	Create {
		#[arg(long)]
		project: String,
		name: String,
		#[arg(long, default_value = "")]
		description: String,
	},
	Show {
		id: String,
	},
	Close {
		id: String,
	},
	Delete {
		id: String,
	},
}

#[derive(Subcommand)]
enum TaskCmd {
	List {
		#[arg(long)]
		project: String,
		#[arg(long)]
		status: Option<String>,
		#[arg(long)]
		epic: Option<String>,
		#[arg(long)]
		assignee: Option<String>,
		#[arg(long)]
		label: Option<String>,
		#[arg(long, name = "type")]
		kind: Option<String>,
		#[arg(long)]
		parent: Option<String>,
	},
	Create {
		#[arg(long)]
		project: String,
		title: String,
		#[arg(long)]
		epic: Option<String>,
		#[arg(long, name = "type", default_value = "task")]
		kind: String,
		#[arg(long)]
		parent: Option<String>,
		#[arg(long, default_value = "medium")]
		priority: String,
		#[arg(long)]
		assignee: Option<String>,
		#[arg(long)]
		label: Vec<String>,
	},
	Show {
		id: String,
	},
	Update {
		id: String,
		#[arg(long)]
		title: Option<String>,
		#[arg(long)]
		description: Option<String>,
		#[arg(long)]
		status: Option<String>,
		#[arg(long)]
		priority: Option<String>,
		#[arg(long)]
		assignee: Option<String>,
		#[arg(long)]
		epic: Option<String>,
		#[arg(long, name = "type")]
		kind: Option<String>,
	},
	Done {
		id: String,
	},
	Log {
		id: String,
		message: String,
	},
	Events {
		id: String,
	},
	Delete {
		id: String,
	},
}

#[derive(Subcommand)]
enum LabelCmd {
	List,
	Create {
		name: String,
		#[arg(long, default_value = "#6b7280")]
		color: String,
	},
	Delete {
		id: String,
	},
}

// Table display types

#[derive(Tabled)]
struct ProjectRow {
	id: String,
	name: String,
	description: String,
}

#[derive(Tabled)]
struct EpicRow {
	id: String,
	name: String,
	status: String,
}

#[derive(Tabled)]
struct TaskRow {
	id: String,
	kind: String,
	title: String,
	status: String,
	priority: String,
	assignee: String,
	labels: String,
}

#[derive(Tabled)]
struct LabelRow {
	id: String,
	name: String,
	color: String,
}


fn main() {
	let cli = Cli::parse();
	let db = Database::open(&cli.db).unwrap_or_else(|e| {
		eprintln!("error: {e}");
		std::process::exit(1);
	});

	let result = run(&db, cli.command);
	if let Err(e) = result {
		eprintln!("error: {e}");
		std::process::exit(1);
	}
}

fn run(db: &Database, cmd: Commands) -> taskd_core::error::Result<()> {
	match cmd {
		Commands::Project { cmd } => match cmd {
			ProjectCmd::List => {
				let projects = db.list_projects()?;
				let rows: Vec<ProjectRow> = projects
					.into_iter()
					.map(|p| ProjectRow {
						id: p.id,
						name: p.name,
						description: p.description,
					})
					.collect();
				println!("{}", Table::new(rows));
			}
			ProjectCmd::Create { name, description } => {
				let p = db.create_project(CreateProject { name, description })?;
				println!("created project {} ({})", p.name, p.id);
			}
			ProjectCmd::Show { id } => {
				let p = db.get_project(&id)?;
				println!("id:          {}", p.id);
				println!("name:        {}", p.name);
				println!("description: {}", p.description);
				println!("created:     {}", p.created_at);
				println!("updated:     {}", p.updated_at);
			}
			ProjectCmd::Delete { id } => {
				db.delete_project(&id)?;
				println!("deleted");
			}
		},
		Commands::Epic { cmd } => match cmd {
			EpicCmd::List { project } => {
				let epics = db.list_epics(&project)?;
				let rows: Vec<EpicRow> = epics
					.into_iter()
					.map(|e| EpicRow {
						id: e.id,
						name: e.name,
						status: e.status,
					})
					.collect();
				println!("{}", Table::new(rows));
			}
			EpicCmd::Create { project, name, description } => {
				let e = db.create_epic(&project, CreateEpic { name, description })?;
				println!("created epic {} ({})", e.name, e.id);
			}
			EpicCmd::Show { id } => {
				let e = db.get_epic(&id)?;
				println!("id:          {}", e.id);
				println!("project:     {}", e.project_id);
				println!("name:        {}", e.name);
				println!("description: {}", e.description);
				println!("status:      {}", e.status);
				println!("created:     {}", e.created_at);
				println!("updated:     {}", e.updated_at);
			}
			EpicCmd::Close { id } => {
				db.update_epic(&id, UpdateEpic { status: Some("closed".into()), ..Default::default() })?;
				println!("closed");
			}
			EpicCmd::Delete { id } => {
				db.delete_epic(&id)?;
				println!("deleted");
			}
		},
		Commands::Task { cmd } => match cmd {
			TaskCmd::List { project, status, epic, assignee, label, kind, parent } => {
				let filter = TaskFilter { status, epic_id: epic, assignee, label, kind, parent_id: parent };
				let tasks = db.list_tasks(&project, &filter)?;
				let rows: Vec<TaskRow> = tasks
					.into_iter()
					.map(|t| TaskRow {
						id: t.id,
						kind: t.kind,
						title: t.title,
						status: t.status,
						priority: t.priority,
						assignee: t.assignee.unwrap_or_default(),
						labels: t.labels.iter().map(|l| l.name.clone()).collect::<Vec<_>>().join(", "),
					})
					.collect();
				println!("{}", Table::new(rows));
			}
			TaskCmd::Create { project, title, epic, kind, parent, priority, assignee, label } => {
				let t = db.create_task(&project, CreateTask {
					title,
					description: String::new(),
					epic_id: epic,
					parent_id: parent,
					kind,
					priority,
					assignee,
					labels: label,
				})?;
				println!("created {} {} ({})", t.kind, t.title, t.id);
			}
			TaskCmd::Show { id } => {
				let t = db.get_task(&id)?;
				println!("id:          {}", t.id);
				println!("project:     {}", t.project_id);
				println!("epic:        {}", t.epic_id);
				if let Some(ref pid) = t.parent_id {
					println!("parent:      {pid}");
				}
				println!("type:        {}", t.kind);
				println!("title:       {}", t.title);
				println!("description: {}", t.description);
				println!("status:      {}", t.status);
				println!("priority:    {}", t.priority);
				println!("assignee:    {}", t.assignee.as_deref().unwrap_or("-"));
				if !t.labels.is_empty() {
					let names: Vec<_> = t.labels.iter().map(|l| l.name.as_str()).collect();
					println!("labels:      {}", names.join(", "));
				}
				if !t.children.is_empty() {
					println!("children:");
					for child in &t.children {
						println!("  {} [{}] {} ({})", child.id, child.kind, child.title, child.status);
					}
				}
				println!("created:     {}", t.created_at);
				println!("updated:     {}", t.updated_at);
			}
			TaskCmd::Update { id, title, description, status, priority, assignee, epic, kind } => {
				let t = db.update_task(&id, UpdateTask {
					title,
					description,
					status,
					priority,
					assignee,
					epic_id: epic,
					kind,
				})?;
				println!("updated {} {} ({})", t.kind, t.title, t.id);
			}
			TaskCmd::Done { id } => {
				db.update_task(&id, UpdateTask { status: Some("done".into()), ..Default::default() })?;
				println!("done");
			}
			TaskCmd::Log { id, message } => {
				let evt = db.log_event(&id, "comment", &message, "{}")?;
				println!("logged comment on {} ({})", id, evt.id);
			}
			TaskCmd::Events { id } => {
				let events = db.list_task_events(&id)?;
				if events.is_empty() {
					println!("no events");
				} else {
					for evt in events {
						println!("[{}] {}: {}", evt.created_at, evt.kind, evt.message);
					}
				}
			}
			TaskCmd::Delete { id } => {
				db.delete_task(&id)?;
				println!("deleted");
			}
		},
		Commands::Label { cmd } => match cmd {
			LabelCmd::List => {
				let labels = db.list_labels()?;
				let rows: Vec<LabelRow> = labels
					.into_iter()
					.map(|l| LabelRow {
						id: l.id,
						name: l.name,
						color: l.color,
					})
					.collect();
				println!("{}", Table::new(rows));
			}
			LabelCmd::Create { name, color } => {
				let l = db.create_label(CreateLabel { name, color })?;
				println!("created label {} ({})", l.name, l.id);
			}
			LabelCmd::Delete { id } => {
				db.delete_label(&id)?;
				println!("deleted");
			}
		},
		Commands::Serve { port, static_dir } => {
			println!("use taskd-server binary to run the server:");
			println!("  taskd-server --port {port}{}", static_dir.map(|d| format!(" --static-dir {d}")).unwrap_or_default());
		}
	}
	Ok(())
}
