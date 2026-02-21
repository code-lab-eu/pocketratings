import { getToken, setToken } from '$lib/auth';
import type { Category, Location, Product, Purchase, Review } from '$lib/types';

const BASE = typeof import.meta.env !== 'undefined' && import.meta.env.PUBLIC_API_BASE_URL != null
	? String(import.meta.env.PUBLIC_API_BASE_URL).replace(/\/$/, '')
	: '';

function ensureAbsolute(path: string): string {
	if (path.startsWith('http')) return path;
	const base = BASE || (typeof window !== 'undefined' ? window.location.origin : '');
	return `${base}${path.startsWith('/') ? path : `/${path}`}`;
}

export interface ApiError {
	error: string;
	message?: string;
}

/** Fetch with Bearer token and X-New-Token handling. */
export async function apiFetch(path: string, init: RequestInit = {}): Promise<Response> {
	const url = ensureAbsolute(path);
	const token = getToken();
	const headers = new Headers(init.headers);
	if (token) {
		headers.set('Authorization', `Bearer ${token}`);
	}
	if (!headers.has('Content-Type') && init.body != null && typeof init.body === 'string') {
		headers.set('Content-Type', 'application/json');
	}
	const res = await fetch(url, { ...init, headers });
	const newToken = res.headers.get('X-New-Token');
	if (newToken) {
		setToken(newToken);
	}
	return res;
}

/** POST JSON and parse JSON response. */
export async function apiPost<T>(path: string, body: unknown): Promise<T> {
	const res = await apiFetch(path, {
		method: 'POST',
		body: JSON.stringify(body),
		headers: { 'Content-Type': 'application/json' }
	});
	const data = (await res.json()) as T | ApiError;
	if (!res.ok) {
		const err = data as ApiError;
		throw new Error(err.message ?? err.error ?? `HTTP ${res.status}`);
	}
	return data as T;
}

/** GET and parse JSON response. */
export async function apiGet<T>(path: string): Promise<T> {
	const res = await apiFetch(path);
	const text = await res.text();
	let data: T | ApiError;
	try {
		data = (text ? JSON.parse(text) : {}) as T | ApiError;
	} catch {
		throw new Error(
			res.ok
				? `Invalid JSON in response from ${path}`
				: `HTTP ${res.status}: response was not JSON${text ? ` (body: ${text.slice(0, 100)}${text.length > 100 ? '…' : ''})` : ''}`
		);
	}
	if (!res.ok) {
		const err = data as ApiError;
		throw new Error(err.message ?? err.error ?? `HTTP ${res.status}`);
	}
	return data as T;
}

/** PATCH JSON and parse JSON response. */
export async function apiPatch<T>(path: string, body: unknown): Promise<T> {
	const res = await apiFetch(path, {
		method: 'PATCH',
		body: JSON.stringify(body),
		headers: { 'Content-Type': 'application/json' }
	});
	const text = await res.text();
	let data: T | ApiError;
	try {
		data = (text ? JSON.parse(text) : {}) as T | ApiError;
	} catch {
		throw new Error(
			res.ok
				? `Invalid JSON in response from ${path}`
				: `HTTP ${res.status}: response was not JSON`
		);
	}
	if (!res.ok) {
		const err = data as ApiError;
		throw new Error(err.message ?? err.error ?? `HTTP ${res.status}`);
	}
	return data as T;
}

/** DELETE; 204 returns void; 200 with body parses JSON. */
export async function apiDelete(path: string): Promise<void> {
	const res = await apiFetch(path, { method: 'DELETE' });
	if (res.status === 204 || res.status === 200) {
		const text = await res.text();
		if (text.trim() === '') return;
		try {
			JSON.parse(text);
		} catch {
			// ignore
		}
		return;
	}
	const text = await res.text();
	let data: ApiError;
	try {
		data = (text ? JSON.parse(text) : {}) as ApiError;
	} catch {
		throw new Error(`HTTP ${res.status}`);
	}
	throw new Error(data.message ?? data.error ?? `HTTP ${res.status}`);
}

export interface LoginResponse {
	token: string;
}

export function login(email: string, password: string): Promise<LoginResponse> {
	return apiPost<LoginResponse>('/api/v1/auth/login', { email, password });
}

export interface MeResponse {
	user_id: string;
	name: string;
}

export function me(): Promise<MeResponse> {
	return apiGet<MeResponse>('/api/v1/me');
}

/** List categories; optional parent_id for filtering. */
export function listCategories(parentId?: string): Promise<Category[]> {
	const path = parentId ? `/api/v1/categories?parent_id=${encodeURIComponent(parentId)}` : '/api/v1/categories';
	return apiGet<Category[]>(path);
}

/** Get a single category by id. */
export function getCategory(id: string): Promise<Category> {
	return apiGet<Category>(`/api/v1/categories/${encodeURIComponent(id)}`);
}

export interface CreateCategoryBody {
	name: string;
	parent_id?: string | null;
}

export function createCategory(body: CreateCategoryBody): Promise<Category> {
	return apiPost<Category>('/api/v1/categories', body);
}

export function updateCategory(id: string, body: { name?: string; parent_id?: string | null }): Promise<Category> {
	return apiPatch<Category>(`/api/v1/categories/${encodeURIComponent(id)}`, body);
}

export function deleteCategory(id: string): Promise<void> {
	return apiDelete(`/api/v1/categories/${encodeURIComponent(id)}`);
}

/** List products; optional category_id and/or q (search). */
export function listProducts(options?: { category_id?: string; q?: string }): Promise<Product[]> {
	const params = new URLSearchParams();
	if (options?.category_id) params.set('category_id', options.category_id);
	if (options?.q) params.set('q', options.q);
	const query = params.toString();
	const path = query ? `/api/v1/products?${query}` : '/api/v1/products';
	return apiGet<Product[]>(path);
}

/** List reviews; no productId = "my reviews" (current user). */
export function listReviews(productId?: string): Promise<Review[]> {
	const path = productId ? `/api/v1/reviews?product_id=${encodeURIComponent(productId)}` : '/api/v1/reviews';
	return apiGet<Review[]>(path);
}

export interface CreateReviewBody {
	product_id: string;
	rating: number;
	text?: string | null;
}

export function getReview(id: string): Promise<Review> {
	return apiGet<Review>(`/api/v1/reviews/${encodeURIComponent(id)}`);
}

export function createReview(body: CreateReviewBody): Promise<Review> {
	return apiPost<Review>('/api/v1/reviews', body);
}

export function updateReview(id: string, body: { rating?: number; text?: string | null }): Promise<Review> {
	return apiPatch<Review>(`/api/v1/reviews/${encodeURIComponent(id)}`, body);
}

export function deleteReview(id: string): Promise<void> {
	return apiDelete(`/api/v1/reviews/${encodeURIComponent(id)}`);
}

/** Get a single product by id. */
export function getProduct(id: string): Promise<Product> {
	return apiGet<Product>(`/api/v1/products/${encodeURIComponent(id)}`);
}

export interface CreateProductBody {
	name: string;
	brand: string;
	category_id: string;
}

export function createProduct(body: CreateProductBody): Promise<Product> {
	return apiPost<Product>('/api/v1/products', body);
}

export function updateProduct(
	id: string,
	body: { name?: string; brand?: string; category_id?: string }
): Promise<Product> {
	return apiPatch<Product>(`/api/v1/products/${encodeURIComponent(id)}`, body);
}

export function deleteProduct(id: string): Promise<void> {
	return apiDelete(`/api/v1/products/${encodeURIComponent(id)}`);
}

/** List purchases; optional product_id for filtering (e.g. purchase history on product detail). */
export function listPurchases(options?: { product_id?: string }): Promise<Purchase[]> {
	const params = new URLSearchParams();
	if (options?.product_id) params.set('product_id', options.product_id);
	const query = params.toString();
	const path = query ? `/api/v1/purchases?${query}` : '/api/v1/purchases';
	return apiGet<Purchase[]>(path);
}

export function getPurchase(id: string): Promise<Purchase> {
	return apiGet<Purchase>(`/api/v1/purchases/${encodeURIComponent(id)}`);
}

export interface CreatePurchaseBody {
	product_id: string;
	location_id: string;
	quantity?: number;
	price?: string;
	purchased_at?: string;
}

export function createPurchase(body: CreatePurchaseBody): Promise<Purchase> {
	return apiPost<Purchase>('/api/v1/purchases', body);
}

export function updatePurchase(
	id: string,
	body: { product_id?: string; location_id?: string; quantity?: number; price?: string; purchased_at?: string }
): Promise<Purchase> {
	return apiPatch<Purchase>(`/api/v1/purchases/${encodeURIComponent(id)}`, body);
}

export function deletePurchase(id: string): Promise<void> {
	return apiDelete(`/api/v1/purchases/${encodeURIComponent(id)}`);
}

/** List all locations (e.g. to resolve location_id to name in purchase history). */
export function listLocations(): Promise<Location[]> {
	return apiGet<Location[]>('/api/v1/locations');
}

/** Get a single location by id. */
export function getLocation(id: string): Promise<Location> {
	return apiGet<Location>(`/api/v1/locations/${encodeURIComponent(id)}`);
}

export function createLocation(body: { name: string }): Promise<Location> {
	return apiPost<Location>('/api/v1/locations', body);
}

export function updateLocation(id: string, body: { name: string }): Promise<Location> {
	return apiPatch<Location>(`/api/v1/locations/${encodeURIComponent(id)}`, body);
}

export function deleteLocation(id: string): Promise<void> {
	return apiDelete(`/api/v1/locations/${encodeURIComponent(id)}`);
}
