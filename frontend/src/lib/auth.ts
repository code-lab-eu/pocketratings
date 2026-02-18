import { get as getStoreValue, writable } from 'svelte/store';

const TOKEN_KEY = 'pocketratings_token';

/** Reactive token store. Updated by setToken/clearToken; sync from localStorage on client init. */
export const token = writable<string | null>(null);

/** Get stored JWT (from store on client, so reactive). */
export function getToken(): string | null {
	if (typeof window === 'undefined') return null;
	const value = getStoreValue(token);
	if (value !== null) return value;
	const fromStorage = localStorage.getItem(TOKEN_KEY);
	if (fromStorage !== null) {
		token.set(fromStorage);
		return fromStorage;
	}
	return null;
}

/** Store JWT and persist to localStorage. Updates the store so the UI re-renders. */
export function setToken(newToken: string): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(TOKEN_KEY, newToken);
	token.set(newToken);
}

/** Remove JWT (e.g. on logout). */
export function clearToken(): void {
	if (typeof window === 'undefined') return;
	localStorage.removeItem(TOKEN_KEY);
	token.set(null);
}
