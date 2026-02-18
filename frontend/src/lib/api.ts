import { getToken, setToken } from '$lib/auth';

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
