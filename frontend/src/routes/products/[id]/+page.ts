import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getCategory, getProduct, listLocations, listPurchases, listReviews } from '$lib/api';

const empty = {
	product: null,
	reviews: [] as Awaited<ReturnType<typeof listReviews>>,
	purchases: [] as Awaited<ReturnType<typeof listPurchases>>,
	category: null,
	locationNames: {} as Record<string, string>
};

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id || !isValidUuid(id)) {
		return { ...empty, notFound: true, error: !id ? 'Missing product id' : null };
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
		return { product, reviews, purchases, category, locationNames, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			...empty,
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
