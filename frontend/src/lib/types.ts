/** Ancestor entry for breadcrumbs (closest parent first). */
export interface CategoryAncestor {
  id: string;
  name: string;
}

/** Category from GET /api/v1/categories (and :id). List returns nested tree via `children`. */
export interface Category {
  id: string;
  /** Breadcrumb trail: closest parent first. */
  ancestors: CategoryAncestor[];
  name: string;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
  /** Nested children (list response only). */
  children?: Category[];
}

/** Product from GET /api/v1/products (and :id). Includes nested category with ancestors. */
export interface Product {
  id: string;
  category: { id: string; name: string; ancestors: CategoryAncestor[] };
  brand: string;
  name: string;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
}

/** Review from GET /api/v1/reviews (and :id). Rating is 1–5. Response includes nested product and user. */
export interface Review {
  id: string;
  product: { id: string; brand: string; name: string };
  user: { id: string; name: string };
  rating: number;
  text: string | null;
  created_at: number;
  updated_at: number;
  deleted_at: number | null;
}

/** One variation in GET /api/v1/products/:id/variations. */
export interface ProductVariation {
  id: string;
  label: string;
  unit: string;
  quantity?: number | null;
  /** Number of purchases referencing this variation (edit-product UI). */
  purchase_count?: number;
}

/** Purchase from GET /api/v1/purchases (and :id). Response includes nested user, product, variation, location. */
export interface Purchase {
  id: string;
  user: { id: string; name: string };
  product: { id: string; brand: string; name: string };
  variation: { id: string; label: string; unit: string; quantity?: number | null };
  location: { id: string; name: string };
  quantity: number;
  price: string;
  purchased_at: number;
  deleted_at: number | null;
}

/** Location from GET /api/v1/locations (and :id). */
export interface Location {
  id: string;
  name: string;
  deleted_at: number | null;
}
