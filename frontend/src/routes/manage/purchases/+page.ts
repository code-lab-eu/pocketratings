import type { PageLoad } from './$types';
import { listLocations, listProducts, listPurchases } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const [purchases, products, locations] = await Promise.all([
			listPurchases(),
			listProducts(),
			listLocations()
		]);
		const productMap = new Map(products.map((p) => [p.id, p]));
		const locationMap = new Map(locations.map((l) => [l.id, l]));
		return { purchases, productMap, locationMap, error: null };
	} catch (e) {
		return {
			purchases: [],
			productMap: new Map(),
			locationMap: new Map(),
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
