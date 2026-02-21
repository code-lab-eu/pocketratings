import type { PageLoad } from './$types';
import { listLocations, listProducts } from '$lib/api';

export const load: PageLoad = async ({ url }) => {
	const productId = url.searchParams.get('product_id') ?? undefined;
	try {
		const [products, locations] = await Promise.all([listProducts(), listLocations()]);
		return { products, locations, productId, error: null };
	} catch (e) {
		return {
			products: [],
			locations: [],
			productId,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
