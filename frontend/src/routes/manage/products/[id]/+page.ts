import type { PageLoad } from './$types';
import { getProduct, listCategories } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id) {
		return { product: null, categories: [], error: 'Missing product id' };
	}
	try {
		const [product, categories] = await Promise.all([getProduct(id), listCategories()]);
		return { product, categories, error: null };
	} catch (e) {
		return {
			product: null,
			categories: [],
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
