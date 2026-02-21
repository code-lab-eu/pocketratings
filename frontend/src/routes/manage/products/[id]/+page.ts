import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getProduct, listCategories } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id || !isValidUuid(id)) {
		return { product: null, categories: [], notFound: true, error: !id ? 'Missing product id' : null };
	}
	try {
		const [product, categories] = await Promise.all([getProduct(id), listCategories()]);
		return { product, categories, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			product: null,
			categories: [],
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
