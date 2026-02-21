import type { PageLoad } from './$types';
import { listProducts, listReviews } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const [reviews, products] = await Promise.all([listReviews(), listProducts()]);
		const productMap = new Map(products.map((p) => [p.id, p]));
		return { reviews, productMap, error: null };
	} catch (e) {
		return {
			reviews: [],
			productMap: new Map(),
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
