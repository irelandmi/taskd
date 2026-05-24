export function el<K extends keyof HTMLElementTagNameMap>(
	tag: K,
	attrs?: Record<string, string>,
	...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
	const e = document.createElement(tag);
	if (attrs) Object.entries(attrs).forEach(([k, v]) => e.setAttribute(k, v));
	children.forEach(c => e.append(typeof c === 'string' ? document.createTextNode(c) : c));
	return e;
}
