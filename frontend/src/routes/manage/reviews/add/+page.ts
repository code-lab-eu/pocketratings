import type { PageLoad } from './$types';
import { listProducts } from '$lib/api';

export const load: PageLoad = async ({ url }) => {
	const productId = url.searchParams.get('product_id') ?? undefined;
	try {
		const products = await listProducts();
		return { products, productId, error: null };
	} catch (e) {
		return { products: [], productId, error: e instanceof Error ? e.message : String(e) };
	}
};
