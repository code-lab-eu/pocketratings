import type { PageLoad } from './$types';
import { getCategory, listCategories } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id) {
		return { category: null, categories: [], error: 'Missing category id' };
	}
	try {
		const [category, categories] = await Promise.all([getCategory(id), listCategories()]);
		return { category, categories, error: null };
	} catch (e) {
		return {
			category: null,
			categories: [],
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
