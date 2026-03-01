import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getCategory, listCategories, listProducts, listReviews } from '$lib/api';
import type { Product, Review } from '$lib/types';

export interface ProductWithReview {
	product: Product;
	rating?: number;
	text?: string | null;
}

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id || !isValidUuid(id)) {
		return { category: null, childCategories: [], items: [], notFound: true, error: !id ? 'Missing category id' : null };
	}
	try {
		const [category, childCategories, products, reviews] = await Promise.all([
			getCategory(id),
			listCategories(id),
			listProducts({ category_id: id }),
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
		return { category, childCategories, items, notFound: false, error: null };
	} catch (e) {
		const notFound = e instanceof ApiClientError && e.status === 404;
		return {
			category: null,
			childCategories: [],
			items: [],
			notFound,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
