import { getToken, setToken } from '$lib/auth';
import type { Category, Product, Review } from '$lib/types';

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
	const data = (await res.json()) as T | ApiError;
	if (!res.ok) {
		const err = data as ApiError;
		throw new Error(err.message ?? err.error ?? `HTTP ${res.status}`);
	}
	return data as T;
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
