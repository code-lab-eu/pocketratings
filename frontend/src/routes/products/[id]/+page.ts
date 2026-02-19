import type { PageLoad } from './$types';
import { getCategory, getProduct, listLocations, listPurchases, listReviews } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id) {
		return { product: null, reviews: [], purchases: [], category: null, locationNames: new Map<string, string>(), error: 'Missing product id' };
	}
	try {
		const product = await getProduct(id);
		const [reviews, purchases, category, locations] = await Promise.all([
			listReviews(id),
			listPurchases({ product_id: id }),
			getCategory(product.category_id),
			listLocations()
		]);
		// Sort reviews most recent first (updated_at descending)
		reviews.sort((a, b) => b.updated_at - a.updated_at);
		const locationNames: Record<string, string> = {};
		for (const loc of locations) {
			locationNames[loc.id] = loc.name;
		}
		return { product, reviews, purchases, category, locationNames, error: null };
	} catch (e) {
		return {
			product: null,
			reviews: [],
			purchases: [],
			category: null,
			locationNames: {} as Record<string, string>,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
