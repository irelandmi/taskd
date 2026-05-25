import { api, type Task, type TaskEvent, type TaskOutput } from '../api';
import { navigate } from '../router';
import { el } from '../dom';

const STATUSES = ['todo', 'in_progress', 'done', 'cancelled', 'blocked'];
const PRIORITIES = ['low', 'medium', 'high', 'urgent'];
const KINDS = ['story', 'task', 'spike', 'bug', 'chore'];

export async function taskDetail(params: Record<string, string>): Promise<HTMLElement> {
	const taskId = params['id'];
	let task: Task;
	try {
		task = await api.tasks.get(taskId);
	} catch {
		return el('div', { class: 'page' }, el('p', {}, 'Task not found.'));
	}

	const [epics, parentTask] = await Promise.all([
		api.epics.list(task.project_id),
		task.parent_id ? api.tasks.get(task.parent_id).catch(() => null) : Promise.resolve(null),
	]);

	const container = el('div', { class: 'page' });
	const header = el('div', { class: 'page-header' },
		el('a', { href: `#/projects/${task.project_id}`, class: 'back' }, '\u2190 Board'),
		el('h1', {}, task.title + ' ', el('span', { class: 'task-id' }, task.id)),
	);
	container.append(header);

	// Parent link
	if (parentTask) {
		const parentLink = el('a', { href: `#/tasks/${parentTask.id}`, class: 'parent-link' },
			'\u2191 Parent: ' + parentTask.title,
		);
		container.append(parentLink);
	}

	const form = el('form', { class: 'detail-form' });

	// Title
	const titleInput = el('input', { type: 'text', value: task.title, name: 'title' });
	form.append(el('label', {}, 'Title'), titleInput);

	// Description
	const descInput = el('textarea', { name: 'description', rows: '4', placeholder: 'Description...' });
	descInput.value = task.description;
	form.append(el('label', {}, 'Description'), descInput);

	// Epic
	const epicSelect = el('select', { name: 'epic_id' }) as HTMLSelectElement;
	for (const epic of epics) {
		const opt = el('option', { value: epic.id }, epic.name) as HTMLOptionElement;
		if (epic.id === task.epic_id) opt.selected = true;
		epicSelect.append(opt);
	}
	form.append(el('label', {}, 'Epic'), epicSelect);

	// Type
	const kindSelect = el('select', { name: 'kind' });
	KINDS.forEach(k => {
		const opt = el('option', { value: k }, k);
		if (k === task.kind) opt.selected = true;
		kindSelect.append(opt);
	});
	form.append(el('label', {}, 'Type'), kindSelect);

	// Status
	const statusSelect = el('select', { name: 'status' });
	STATUSES.forEach(s => {
		const opt = el('option', { value: s }, s.replace('_', ' '));
		if (s === task.status) opt.selected = true;
		statusSelect.append(opt);
	});
	form.append(el('label', {}, 'Status'), statusSelect);

	// Priority
	const prioritySelect = el('select', { name: 'priority' });
	PRIORITIES.forEach(p => {
		const opt = el('option', { value: p }, p);
		if (p === task.priority) opt.selected = true;
		prioritySelect.append(opt);
	});
	form.append(el('label', {}, 'Priority'), prioritySelect);

	// Assignee
	const assigneeInput = el('input', { type: 'text', value: task.assignee || '', name: 'assignee', placeholder: 'Assignee...' });
	form.append(el('label', {}, 'Assignee'), assigneeInput);

	// Labels
	if (task.labels.length > 0) {
		const labelRow = el('div', { class: 'task-labels' });
		task.labels.forEach(l => {
			labelRow.append(el('span', { class: 'label-badge', style: `background:${l.color}` }, l.name));
		});
		form.append(el('label', {}, 'Labels'), labelRow);
	}

	// Children
	const childSection = el('div', { class: 'children-section' });
	const childLabel = el('label', {}, `Sub-tasks (${task.children.length})`);
	const childList = el('div', { class: 'children-list' });
	childSection.append(childLabel, childList);
	form.append(childSection);

	function renderChildRow(child: Task) {
		const row = el('div', { class: 'child-row', style: 'cursor:pointer;padding:4px 0;display:flex;gap:8px;align-items:center' },
			el('span', { class: 'badge', style: 'font-size:0.75em' }, child.kind),
			el('span', {}, child.title),
			el('span', { class: `badge status-${child.status}`, style: 'font-size:0.75em' }, child.status),
		);
		row.addEventListener('click', () => navigate(`#/tasks/${child.id}`));
		childList.append(row);
	}

	for (const child of task.children) {
		renderChildRow(child);
	}

	// Inline sub-task creation
	const subForm = el('div', { class: 'inline-form', style: 'margin-top:8px' });
	const subInput = el('input', { type: 'text', placeholder: 'Add sub-task...', style: 'flex:1' }) as HTMLInputElement;
	const subBtn = el('button', { type: 'button' }, 'Add Sub-task');
	subForm.append(subInput, subBtn);
	form.append(subForm);

	let childCount = task.children.length;

	subBtn.addEventListener('click', async () => {
		const title = subInput.value.trim();
		if (!title) return;
		const newChild = await api.tasks.create(task.project_id, {
			title,
			epic_id: task.epic_id,
			parent_id: task.id,
		});
		subInput.value = '';
		childCount++;
		childLabel.textContent = `Sub-tasks (${childCount})`;
		renderChildRow(newChild);
	});

	// Meta
	form.append(el('div', { class: 'meta' }, `Created: ${task.created_at} | Updated: ${task.updated_at}`));

	const saveBtn = el('button', { type: 'submit' }, 'Save');
	const deleteBtn = el('button', { type: 'button', class: 'danger' }, 'Delete');
	const actions = el('div', { class: 'form-actions' });
	actions.append(saveBtn, deleteBtn);
	form.append(actions);

	form.addEventListener('submit', async (e) => {
		e.preventDefault();
		await api.tasks.update(task.id, {
			title: titleInput.value,
			description: descInput.value,
			epic_id: epicSelect.value,
			kind: kindSelect.value,
			status: statusSelect.value,
			priority: prioritySelect.value,
			assignee: assigneeInput.value || null,
		});
		window.location.hash = `#/projects/${task.project_id}`;
	});

	deleteBtn.addEventListener('click', async () => {
		if (confirm('Delete this task?')) {
			await api.tasks.delete(task.id);
			window.location.hash = `#/projects/${task.project_id}`;
		}
	});

	container.append(form);

	// --- Outputs Section ---
	const outputSection = el('div', { class: 'outputs-section' });
	outputSection.append(el('h2', {}, 'Outputs'));

	const outputList = el('div', { class: 'output-list' });
	outputSection.append(outputList);

	function renderOutput(o: TaskOutput) {
		const kindIcon = { file: '\u{1F4C4}', commit: '\u{1F517}', url: '\u{1F310}', text: '\u{1F4DD}' }[o.kind] || '';
		const entry = el('div', { class: `output-entry output-${o.kind}` },
			el('span', { class: 'badge' }, o.kind),
			el('span', { class: 'output-ref' }, `${kindIcon} ${o.reference}`),
			o.label ? el('span', { class: 'output-label' }, o.label) : el('span', {}),
		);
		outputList.append(entry);
	}

	if (task.outputs.length === 0) {
		outputList.append(el('p', { class: 'empty' }, 'No outputs yet.'));
	} else {
		task.outputs.forEach(renderOutput);
	}

	const outputForm = el('div', { class: 'inline-form', style: 'margin-top:8px' });
	const outputKind = el('select', {}) as HTMLSelectElement;
	['file', 'commit', 'url', 'text'].forEach(k => {
		outputKind.append(el('option', { value: k }, k));
	});
	const outputRef = el('input', { type: 'text', placeholder: 'Reference (path, SHA, URL, text)...' }) as HTMLInputElement;
	const outputLabel = el('input', { type: 'text', placeholder: 'Label (optional)...' }) as HTMLInputElement;
	const outputBtn = el('button', { type: 'button' }, 'Add Output');
	outputForm.append(outputKind, outputRef, outputLabel, outputBtn);
	outputSection.append(outputForm);

	outputBtn.addEventListener('click', async () => {
		const ref = outputRef.value.trim();
		if (!ref) return;
		const o = await api.tasks.addOutput(task.id, {
			kind: outputKind.value,
			reference: ref,
			label: outputLabel.value.trim(),
		});
		outputRef.value = '';
		outputLabel.value = '';
		if (outputList.querySelector('.empty')) outputList.replaceChildren();
		renderOutput(o);
	});

	container.append(outputSection);

	// --- Dependencies Section ---
	const depSection = el('div', { class: 'dependencies-section' });
	depSection.append(el('h2', {}, 'Dependencies'));

	const depList = el('div', { class: 'dependency-list' });
	depSection.append(depList);

	async function loadDependencies() {
		depList.replaceChildren();
		if (task.dependencies.length === 0) {
			depList.append(el('p', { class: 'empty' }, 'No dependencies.'));
			return;
		}
		for (const depId of task.dependencies) {
			try {
				const dep = await api.tasks.get(depId);
				const row = el('div', { class: 'dependency-row' },
					el('span', { class: `badge status-${dep.status}` }, dep.status),
					el('a', { href: `#/tasks/${dep.id}` }, dep.title),
					el('span', { class: 'task-id' }, dep.id),
				);
				const removeBtn = el('button', { type: 'button', class: 'danger', style: 'padding:2px 6px;font-size:11px' }, 'x');
				removeBtn.addEventListener('click', async () => {
					await api.tasks.removeDependency(task.id, dep.id);
					task.dependencies = task.dependencies.filter(d => d !== dep.id);
					loadDependencies();
				});
				row.append(removeBtn);
				depList.append(row);
			} catch {
				depList.append(el('div', { class: 'dependency-row' }, el('span', {}, depId)));
			}
		}
	}
	await loadDependencies();

	container.append(depSection);

	// --- Activity Log ---
	const logSection = el('div', { class: 'activity-log' });
	logSection.append(el('h2', {}, 'Activity'));

	const logList = el('div', { class: 'log-list' });
	logSection.append(logList);

	function renderEvent(evt: TaskEvent) {
		const kindClass = `event-${evt.kind}`;
		const entry = el('div', { class: `log-entry ${kindClass}` },
			el('span', { class: 'log-time' }, evt.created_at.replace('T', ' ').replace('Z', '')),
			el('span', { class: `badge event-kind-badge` }, evt.kind),
			el('span', { class: 'log-message' }, evt.message),
		);
		logList.append(entry);
	}

	const events = await api.tasks.events(task.id);
	if (events.length === 0) {
		logList.append(el('p', { class: 'empty' }, 'No activity yet.'));
	} else {
		events.forEach(renderEvent);
	}

	// Comment form
	const commentForm = el('div', { class: 'inline-form', style: 'margin-top:8px' });
	const commentInput = el('input', { type: 'text', placeholder: 'Add a comment...' }) as HTMLInputElement;
	const commentBtn = el('button', { type: 'button' }, 'Comment');
	commentForm.append(commentInput, commentBtn);
	logSection.append(commentForm);

	commentBtn.addEventListener('click', async () => {
		const message = commentInput.value.trim();
		if (!message) return;
		const evt = await api.tasks.addComment(task.id, message);
		commentInput.value = '';
		if (logList.querySelector('.empty')) logList.replaceChildren();
		renderEvent(evt);
	});

	container.append(logSection);

	// SSE live updates
	const evtSource = new EventSource('/api/events');
	evtSource.onmessage = async (e) => {
		try {
			const data = JSON.parse(e.data);
			if (data.task_id === task.id && data.type === 'task_event') {
				const freshEvents = await api.tasks.events(task.id);
				logList.replaceChildren();
				freshEvents.forEach(renderEvent);
			}
		} catch {}
	};
	const cleanup = () => {
		evtSource.close();
		window.removeEventListener('hashchange', cleanup);
	};
	window.addEventListener('hashchange', cleanup);

	return container;
}
