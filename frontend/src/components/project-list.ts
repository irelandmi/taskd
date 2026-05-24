import { api, type Project, type Task } from '../api';
import { navigate } from '../router';
import { el } from '../dom';

interface ProjectStats {
	project: Project;
	tasks: Task[];
	todo: number;
	in_progress: number;
	done: number;
	cancelled: number;
	total: number;
}

async function loadStats(): Promise<ProjectStats[]> {
	const projects = await api.projects.list();
	return Promise.all(projects.map(async (project) => {
		const tasks = await api.tasks.list(project.id);
		return {
			project,
			tasks,
			todo: tasks.filter(t => t.status === 'todo').length,
			in_progress: tasks.filter(t => t.status === 'in_progress').length,
			done: tasks.filter(t => t.status === 'done').length,
			cancelled: tasks.filter(t => t.status === 'cancelled').length,
			total: tasks.length,
		};
	}));
}

function progressBar(stats: ProjectStats): HTMLElement {
	const bar = el('div', { class: 'progress-bar' });
	if (stats.total === 0) return bar;
	const segments: [number, string][] = [
		[stats.done, 'var(--green)'],
		[stats.in_progress, 'var(--accent)'],
		[stats.todo, 'var(--border)'],
		[stats.cancelled, 'var(--muted)'],
	];
	for (const [count, color] of segments) {
		if (count === 0) continue;
		const pct = (count / stats.total) * 100;
		bar.append(el('div', { class: 'progress-segment', style: `width:${pct}%;background:${color}` }));
	}
	return bar;
}

export async function projectList(): Promise<HTMLElement> {
	const container = el('div', { class: 'page' });

	const header = el('div', { class: 'page-header' },
		el('h1', {}, 'Overview'),
	);

	const form = el('form', { class: 'inline-form' });
	const input = el('input', { type: 'text', placeholder: 'New project name...', required: '' });
	const btn = el('button', { type: 'submit' }, 'Create');
	form.append(input, btn);
	header.append(form);
	container.append(header);

	const list = el('div', { class: 'card-grid' });
	container.append(list);

	async function load() {
		const stats = await loadStats();
		list.replaceChildren();
		if (stats.length === 0) {
			list.append(el('p', { class: 'empty' }, 'No projects yet.'));
			return;
		}
		for (const s of stats) {
			const counters = el('div', { class: 'stat-row' },
				el('span', { class: 'stat stat-todo' }, `${s.todo} todo`),
				el('span', { class: 'stat stat-progress' }, `${s.in_progress} active`),
				el('span', { class: 'stat stat-done' }, `${s.done} done`),
			);
			const card = el('div', { class: 'card' },
				el('h3', {}, s.project.name),
				el('p', { class: 'muted' }, s.project.description || 'No description'),
				progressBar(s),
				counters,
				el('span', { class: 'meta' }, `${s.total} tasks`),
			);
			card.addEventListener('click', () => navigate(`#/projects/${s.project.id}`));
			list.append(card);
		}
	}

	form.addEventListener('submit', async (e) => {
		e.preventDefault();
		const name = input.value.trim();
		if (!name) return;
		await api.projects.create({ name });
		input.value = '';
		await load();
	});

	await load();
	return container;
}
