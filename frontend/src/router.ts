type RouteHandler = (params: Record<string, string>) => HTMLElement | Promise<HTMLElement>;

interface Route {
	pattern: RegExp;
	keys: string[];
	handler: RouteHandler;
}

const routes: Route[] = [];

export function route(pattern: string, handler: RouteHandler) {
	const keys: string[] = [];
	const regex = pattern.replace(/:(\w+)/g, (_match, key) => {
		keys.push(key);
		return '([^/]+)';
	});
	routes.push({ pattern: new RegExp(`^${regex}$`), keys, handler });
}

function parseHash(): { path: string; query: Record<string, string> } {
	const raw = window.location.hash.slice(1) || '/';
	const [path, qs] = raw.split('?', 2);
	const query: Record<string, string> = {};
	if (qs) {
		new URLSearchParams(qs).forEach((v, k) => { query[k] = v; });
	}
	return { path, query };
}

export async function navigate(hash?: string) {
	if (hash !== undefined) {
		window.location.hash = hash;
		return;
	}
	const { path, query } = parseHash();
	const app = document.getElementById('app')!;

	for (const r of routes) {
		const match = path.match(r.pattern);
		if (match) {
			const params: Record<string, string> = { ...query };
			r.keys.forEach((key, i) => {
				params[key] = match[i + 1];
			});
			const elem = await r.handler(params);
			app.replaceChildren(elem);
			return;
		}
	}

	app.replaceChildren(document.createTextNode('404 \u2014 not found'));
}

export function startRouter() {
	window.addEventListener('hashchange', () => navigate());
	navigate();
}
