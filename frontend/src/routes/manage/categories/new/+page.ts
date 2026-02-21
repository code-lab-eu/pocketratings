import type { PageLoad } from './$types';
import { listCategories } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const categories = await listCategories();
		return { categories, error: null };
	} catch (e) {
		return { categories: [], error: e instanceof Error ? e.message : String(e) };
	}
};
