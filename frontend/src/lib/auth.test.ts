import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { clearToken, getToken, setToken, token } from './auth';

const TOKEN_KEY = 'pocketratings_token';

describe('auth', () => {
	beforeEach(() => {
		clearToken();
	});

	afterEach(() => {
		clearToken();
	});

	it('setToken sets localStorage and the token store', () => {
		const t = 'eyJhbGciOiJIUzI1NiJ9.test';
		setToken(t);
		expect(localStorage.getItem(TOKEN_KEY)).toBe(t);
		let value: string | null = null;
		token.subscribe((v) => (value = v))();
		expect(value).toBe(t);
	});

	it('clearToken removes from localStorage and clears the store', () => {
		setToken('some-token');
		clearToken();
		expect(localStorage.getItem(TOKEN_KEY)).toBeNull();
		let value: string | null = undefined as unknown as null;
		token.subscribe((v) => (value = v))();
		expect(value).toBeNull();
	});

	it('getToken returns stored value from store', () => {
		setToken('stored');
		expect(getToken()).toBe('stored');
	});

	it('getToken syncs from localStorage when store is empty', () => {
		localStorage.setItem(TOKEN_KEY, 'from-storage');
		// Store is still null until getToken runs
		expect(getToken()).toBe('from-storage');
		// After getToken, store was updated
		let value: string | null = null;
		token.subscribe((v) => (value = v))();
		expect(value).toBe('from-storage');
	});
});
