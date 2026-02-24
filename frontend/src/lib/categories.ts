import type { Category } from '$lib/types';

/**
 * Flatten a tree of categories to a list with depth for dropdowns or flat lists with indent.
 * Order: parent before children, depth-first.
 */
export function flattenCategories(
	tree: Category[],
	depth = 0
): { category: Category; depth: number }[] {
	const out: { category: Category; depth: number }[] = [];
	for (const c of tree) {
		out.push({ category: c, depth });
		const children = c.children ?? [];
		if (children.length > 0) {
			out.push(...flattenCategories(children, depth + 1));
		}
	}
	return out;
}
