import { api, type Task, type Epic, type Project } from '../api';
import { navigate } from '../router';
import { el } from '../dom';

const STATUSES = ['todo', 'in_progress', 'done', 'cancelled'] as const;
const STATUS_LABELS: Record<string, string> = {
	todo: 'To Do',
	in_progress: 'In Progress',
	done: 'Done',
	cancelled: 'Cancelled',
};

const PRIORITY_CLASSES: Record<string, string> = {
	urgent: 'priority-urgent',
	high: 'priority-high',
	medium: 'priority-medium',
	low: 'priority-low',
};

function kindBadge(kind: string): HTMLElement {
	return el('span', { class: `badge kind-${kind}` }, kind);
}

function taskCard(task: Task, epicMap: Map<string, Epic>, taskMap: Map<string, Task>): HTMLElement {
	const card = el('div', { class: `task-card ${PRIORITY_CLASSES[task.priority] || ''}` },
		el('div', { class: 'task-title' },
			kindBadge(task.kind),
			document.createTextNode(' ' + task.title + ' '),
			el('span', { class: 'task-id' }, task.id),
		),
		el('div', { class: 'task-meta' },
			el('span', { class: `badge priority-badge-${task.priority}` }, task.priority),
			...(task.assignee ? [el('span', { class: 'assignee' }, task.assignee)] : []),
			...(task.children && task.children.length > 0
				? [el('span', { class: 'badge' }, `${task.children.length} sub`)]
				: []),
		),
	);
	const contextRow = el('div', { class: 'task-context' });
	const epic = epicMap.get(task.epic_id);
	if (epic) {
		contextRow.append(el('span', { class: 'badge epic-badge' }, epic.name));
	}
	if (task.parent_id) {
		const parent = taskMap.get(task.parent_id);
		contextRow.append(el('span', { class: 'badge parent-badge' }, '\u2191 ' + (parent ? parent.title : task.parent_id.slice(0, 8))));
	}
	if (contextRow.childElementCount > 0) card.append(contextRow);
	if (task.labels.length > 0) {
		const labelRow = el('div', { class: 'task-labels' });
		task.labels.forEach(l => {
			labelRow.append(el('span', { class: 'label-badge', style: `background:${l.color}` }, l.name));
		});
		card.append(labelRow);
	}
	card.addEventListener('click', () => navigate(`#/tasks/${task.id}`));
	return card;
}

function renderEpicBoard(epic: Epic, tasks: Task[], taskMap: Map<string, Task>, epicMap: Map<string, Epic>): HTMLElement {
	const section = el('div', { class: 'epic-section' });
	const header = el('div', { class: 'epic-section-header' },
		el('h2', {}, epic.name),
		el('span', { class: 'badge' }, `${tasks.length} tasks`),
	);
	section.append(header);

	const board = el('div', { class: 'board' });
	for (const status of STATUSES) {
		const col = el('div', { class: 'board-column' },
			el('h3', {}, STATUS_LABELS[status]),
		);
		tasks.filter(t => t.status === status).forEach(t => col.append(taskCard(t, epicMap, taskMap)));
		board.append(col);
	}
	section.append(board);
	return section;
}

function renderKanban(tasks: Task[], epics: Epic[], epicMap: Map<string, Epic>, taskMap: Map<string, Task>, target: HTMLElement) {
	target.replaceChildren();
	target.className = 'boards';

	// Backlog first, then the rest
	const sorted = [...epics].sort((a, b) => {
		if (a.name === 'Backlog') return -1;
		if (b.name === 'Backlog') return 1;
		return a.created_at.localeCompare(b.created_at);
	});

	for (const epic of sorted) {
		const epicTasks = tasks.filter(t => t.epic_id === epic.id);
		if (epicTasks.length === 0 && epic.status === 'closed') continue;
		target.append(renderEpicBoard(epic, epicTasks, taskMap, epicMap));
	}
}

function renderTimeline(tasks: Task[], epicMap: Map<string, Epic>, taskMap: Map<string, Task>, target: HTMLElement) {
	target.replaceChildren();
	target.className = 'timeline';

	const sorted = [...tasks].sort((a, b) => a.created_at.localeCompare(b.created_at));

	// Group by date
	const groups = new Map<string, Task[]>();
	for (const task of sorted) {
		const date = task.created_at.split('T')[0];
		if (!groups.has(date)) groups.set(date, []);
		groups.get(date)!.push(task);
	}

	if (groups.size === 0) {
		target.append(el('p', { class: 'empty' }, 'No tasks yet.'));
		return;
	}

	for (const [date, dateTasks] of groups) {
		const group = el('div', { class: 'timeline-group' });
		const marker = el('div', { class: 'timeline-date' },
			el('span', { class: 'timeline-dot' }),
			el('span', {}, date),
		);
		group.append(marker);

		const taskList = el('div', { class: 'timeline-tasks' });
		for (const task of dateTasks) {
			const statusClass = `status-${task.status}`;
			const epic = epicMap.get(task.epic_id);
			const parent = task.parent_id ? taskMap.get(task.parent_id) : null;
			const row = el('div', { class: `timeline-task ${PRIORITY_CLASSES[task.priority] || ''}` },
				el('span', { class: `timeline-status ${statusClass}` }, STATUS_LABELS[task.status]),
				kindBadge(task.kind),
				el('span', { class: 'timeline-task-title' }, task.title + ' '),
				el('span', { class: 'task-id' }, task.id),
				el('div', { class: 'task-meta' },
					el('span', { class: `badge priority-badge-${task.priority}` }, task.priority),
					...(epic ? [el('span', { class: 'badge epic-badge' }, epic.name)] : []),
					...(task.parent_id ? [el('span', { class: 'badge parent-badge' }, '\u2191 ' + (parent ? parent.title : task.parent_id.slice(0, 8)))] : []),
					...(task.assignee ? [el('span', { class: 'assignee' }, task.assignee)] : []),
				),
			);
			if (task.labels.length > 0) {
				const labelRow = el('div', { class: 'task-labels' });
				task.labels.forEach(l => {
					labelRow.append(el('span', { class: 'label-badge', style: `background:${l.color}` }, l.name));
				});
				row.append(labelRow);
			}
			row.addEventListener('click', () => navigate(`#/tasks/${task.id}`));
			row.style.cursor = 'pointer';
			taskList.append(row);
		}
		group.append(taskList);
		target.append(group);
	}
}

export async function taskBoard(params: Record<string, string>): Promise<HTMLElement> {
	const projectId = params['id'];
	const initialView = params['view'] || 'kanban';

	let project: Project;
	try {
		project = await api.projects.get(projectId);
	} catch {
		return el('div', { class: 'page' }, el('p', {}, 'Project not found.'));
	}

	let currentView = initialView;
	const container = el('div', { class: 'page' });

	const header = el('div', { class: 'page-header' },
		el('a', { href: '#/', class: 'back' }, '\u2190 Overview'),
		el('h1', {}, project.name),
	);

	// View toggle
	const viewToggle = el('div', { class: 'view-toggle' });
	const kanbanBtn = el('button', { class: 'toggle-btn active', 'data-view': 'kanban' }, 'Kanban');
	const timelineBtn = el('button', { class: 'toggle-btn', 'data-view': 'timeline' }, 'Timeline');
	viewToggle.append(kanbanBtn, timelineBtn);
	header.append(viewToggle);

	const form = el('form', { class: 'inline-form' });
	const input = el('input', { type: 'text', placeholder: 'New task...', required: '' }) as HTMLInputElement;
	const btn = el('button', { type: 'submit' }, 'Add');
	form.append(input, btn);

	const epicForm = el('div', { class: 'inline-form' });
	const epicInput = el('input', { type: 'text', placeholder: 'New epic...' }) as HTMLInputElement;
	const epicBtn = el('button', { type: 'button' }, 'Add Epic');
	epicForm.append(epicInput, epicBtn);

	const forms = el('div', { class: 'header-forms' });
	forms.append(form, epicForm);
	header.append(forms);
	container.append(header);

	const content = el('div', {});
	container.append(content);

	function updateToggle() {
		kanbanBtn.classList.toggle('active', currentView === 'kanban');
		timelineBtn.classList.toggle('active', currentView === 'timeline');
	}

	async function load() {
		const [tasks, epics] = await Promise.all([
			api.tasks.list(projectId),
			api.epics.list(projectId),
		]);
		const epicMap = new Map(epics.map(e => [e.id, e]));
		const taskMap = new Map(tasks.map(t => [t.id, t]));
		if (currentView === 'timeline') {
			renderTimeline(tasks, epicMap, taskMap, content);
		} else {
			renderKanban(tasks, epics, epicMap, taskMap, content);
		}
	}

	kanbanBtn.addEventListener('click', () => {
		currentView = 'kanban';
		updateToggle();
		load();
	});

	timelineBtn.addEventListener('click', () => {
		currentView = 'timeline';
		updateToggle();
		load();
	});

	form.addEventListener('submit', async (e) => {
		e.preventDefault();
		const title = input.value.trim();
		if (!title) return;
		await api.tasks.create(projectId, { title });
		input.value = '';
		await load();
	});

	epicBtn.addEventListener('click', async () => {
		const name = epicInput.value.trim();
		if (!name) return;
		await api.epics.create(projectId, { name });
		epicInput.value = '';
		await load();
	});

	if (initialView === 'timeline') {
		currentView = 'timeline';
		updateToggle();
	}

	await load();

	// SSE live updates
	const evtSource = new EventSource('/api/events');
	evtSource.onmessage = (e) => {
		try {
			const data = JSON.parse(e.data);
			if (!data.project_id || data.project_id === projectId) {
				load();
			}
		} catch {}
	};

	// Cleanup on navigation
	const cleanup = () => {
		evtSource.close();
		window.removeEventListener('hashchange', cleanup);
	};
	window.addEventListener('hashchange', cleanup);

	return container;
}
