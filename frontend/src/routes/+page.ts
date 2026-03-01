import type { PageLoad } from './$types';
import { listCategories, listProducts, listReviews } from '$lib/api';
import { flattenCategories } from '$lib/categories';
import type { Product, Review } from '$lib/types';

export interface ProductWithReview {
	product: Product;
	rating?: number;
	text?: string | null;
}

export const load: PageLoad = async ({ url }) => {
	const q = url.searchParams.get('q') ?? '';
	try {
		const [categoriesTree, products, reviews] = await Promise.all([
			listCategories(),
			q ? listProducts({ q }) : listProducts(),
			listReviews()
		]);
		const flat = flattenCategories(categoriesTree);
		const categories =
			q.trim() === ''
				? flat
				: flat.filter(({ category }) =>
						category.name.toLowerCase().includes(q.toLowerCase())
					);
		// Latest review per product
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
		return { categories, items, query: q, error: null };
	} catch (e) {
		return {
			categories: [],
			items: [],
			query: q,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
