import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getCategory, listCategories } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id || !isValidUuid(id)) {
		return { category: null, categories: [], notFound: true, error: !id ? 'Missing category id' : null };
	}
	try {
		const [category, categories] = await Promise.all([getCategory(id), listCategories()]);
		return { category, categories, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			category: null,
			categories: [],
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
