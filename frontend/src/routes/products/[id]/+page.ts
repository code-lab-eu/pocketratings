import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getProduct, listPurchases, listReviews } from '$lib/api';

const empty = {
	product: null,
	reviews: [] as Awaited<ReturnType<typeof listReviews>>,
	purchases: [] as Awaited<ReturnType<typeof listPurchases>>
};

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id || !isValidUuid(id)) {
		return { ...empty, notFound: true, error: !id ? 'Missing product id' : null };
	}
	try {
		const [product, reviews, purchases] = await Promise.all([
			getProduct(id),
			listReviews(id),
			listPurchases({ product_id: id })
		]);
		// Sort reviews most recent first (updated_at descending)
		reviews.sort((a, b) => b.updated_at - a.updated_at);
		return { product, reviews, purchases, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			...empty,
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
