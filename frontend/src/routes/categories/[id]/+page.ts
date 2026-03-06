import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getCategory, listProducts, listReviews } from '$lib/api';
import type { Product, Review } from '$lib/types';

export interface ProductWithReview {
	product: Product;
	rating?: number;
	text?: string | null;
}

export const load: PageLoad = async ({ params, url }) => {
	const id = params.id;
	const q = url.searchParams.get('q') ?? '';
	if (!id || !isValidUuid(id)) {
		return { category: null, items: [], query: q, notFound: true, error: !id ? 'Missing category id' : null };
	}
	try {
		const [category, products, reviews] = await Promise.all([
			getCategory(id),
			listProducts({ category_id: id, ...(q.trim() && { q }) }),
			listReviews()
		]);
		// Latest review per product (if multiple)
		const reviewByProductId = new Map<string, Review>();
		for (const r of reviews) {
			const existing = reviewByProductId.get(r.product.id);
			if (!existing || r.updated_at > existing.updated_at) {
				reviewByProductId.set(r.product.id, r);
			}
		}
		const items: ProductWithReview[] = products.map((product) => {
			const review = reviewByProductId.get(product.id);
			return {
				product,
				rating: review != null ? review.rating : undefined,
				text: review?.text ?? undefined
			};
		});
		return { category, items, query: q, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			category: null,
			items: [],
			query: q,
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
