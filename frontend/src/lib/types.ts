/** Category from GET /api/v1/categories (and :id). */
export interface Category {
	id: string;
	parent_id: string | null;
	name: string;
	created_at: number;
	updated_at: number;
	deleted_at: number | null;
}

/** Product from GET /api/v1/products (and :id). */
export interface Product {
	id: string;
	category_id: string;
	brand: string;
	name: string;
	created_at: number;
	updated_at: number;
	deleted_at: number | null;
}

/** Review from GET /api/v1/reviews (and :id). Rating is 1–5. */
export interface Review {
	id: string;
	product_id: string;
	user_id: string;
	rating: number;
	text: string | null;
	created_at: number;
	updated_at: number;
	deleted_at: number | null;
}
