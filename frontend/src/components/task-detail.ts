import { api, type Task } from '../api';
import { navigate } from '../router';
import { el } from '../dom';

const STATUSES = ['todo', 'in_progress', 'done', 'cancelled'];
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
	return container;
}
