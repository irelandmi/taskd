const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...options,
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || res.statusText);
	}
	if (res.status === 204) return undefined as T;
	return res.json();
}

export interface Project {
	id: string;
	name: string;
	description: string;
	created_at: string;
	updated_at: string;
}

export interface Epic {
	id: string;
	project_id: string;
	name: string;
	description: string;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface Task {
	id: string;
	project_id: string;
	epic_id: string;
	parent_id: string | null;
	kind: string;
	title: string;
	description: string;
	status: string;
	priority: string;
	assignee: string | null;
	labels: Label[];
	children: Task[];
	outputs: TaskOutput[];
	dependencies: string[];
	created_at: string;
	updated_at: string;
}

export interface TaskOutput {
	id: string;
	task_id: string;
	kind: string;
	reference: string;
	label: string;
	created_at: string;
}

export interface Label {
	id: string;
	name: string;
	color: string;
}

export interface TaskEvent {
	id: string;
	task_id: string;
	kind: string;
	message: string;
	meta: string;
	created_at: string;
}

export const api = {
	projects: {
		list: () => request<Project[]>('/projects'),
		create: (data: { name: string; description?: string }) =>
			request<Project>('/projects', { method: 'POST', body: JSON.stringify(data) }),
		get: (id: string) => request<Project>(`/projects/${id}`),
		update: (id: string, data: Partial<Project>) =>
			request<Project>(`/projects/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
		delete: (id: string) => request<void>(`/projects/${id}`, { method: 'DELETE' }),
	},
	epics: {
		list: (projectId: string) => request<Epic[]>(`/projects/${projectId}/epics`),
		create: (projectId: string, data: { name: string; description?: string }) =>
			request<Epic>(`/projects/${projectId}/epics`, { method: 'POST', body: JSON.stringify(data) }),
		get: (id: string) => request<Epic>(`/epics/${id}`),
		update: (id: string, data: Partial<Epic>) =>
			request<Epic>(`/epics/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
		delete: (id: string) => request<void>(`/epics/${id}`, { method: 'DELETE' }),
	},
	tasks: {
		list: (projectId: string, filters?: Record<string, string>) => {
			const params = new URLSearchParams(filters);
			const qs = params.toString();
			return request<Task[]>(`/projects/${projectId}/tasks${qs ? '?' + qs : ''}`);
		},
		create: (projectId: string, data: { title: string; description?: string; epic_id?: string; parent_id?: string; kind?: string; priority?: string; assignee?: string; labels?: string[] }) =>
			request<Task>(`/projects/${projectId}/tasks`, { method: 'POST', body: JSON.stringify(data) }),
		get: (id: string) => request<Task>(`/tasks/${id}`),
		update: (id: string, data: Partial<Task>) =>
			request<Task>(`/tasks/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
		delete: (id: string) => request<void>(`/tasks/${id}`, { method: 'DELETE' }),
		setLabels: (id: string, labelIds: string[]) =>
			request<Task>(`/tasks/${id}/labels`, { method: 'PUT', body: JSON.stringify(labelIds) }),
		events: (id: string) => request<TaskEvent[]>(`/tasks/${id}/events`),
		addComment: (id: string, message: string) =>
			request<TaskEvent>(`/tasks/${id}/events`, { method: 'POST', body: JSON.stringify({ message }) }),
		outputs: (id: string) => request<TaskOutput[]>(`/tasks/${id}/outputs`),
		addOutput: (id: string, data: { kind: string; reference: string; label?: string }) =>
			request<TaskOutput>(`/tasks/${id}/outputs`, { method: 'POST', body: JSON.stringify(data) }),
		addDependency: (id: string, dependsOn: string) =>
			request<void>(`/tasks/${id}/dependencies`, { method: 'POST', body: JSON.stringify({ depends_on: dependsOn }) }),
		removeDependency: (id: string, depId: string) =>
			request<void>(`/tasks/${id}/dependencies/${depId}`, { method: 'DELETE' }),
	},
	labels: {
		list: () => request<Label[]>('/labels'),
		create: (data: { name: string; color?: string }) =>
			request<Label>('/labels', { method: 'POST', body: JSON.stringify(data) }),
		delete: (id: string) => request<void>(`/labels/${id}`, { method: 'DELETE' }),
	},
};
