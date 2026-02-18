import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as auth from './auth';
import { login } from './api';

describe('api', () => {
	beforeEach(() => {
		vi.stubGlobal('fetch', vi.fn());
		vi.spyOn(auth, 'setToken').mockImplementation(() => {});
		vi.spyOn(auth, 'getToken').mockReturnValue(null);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
		vi.restoreAllMocks();
	});

	it('login sends POST to /api/v1/auth/login with JSON body and returns token on 200', async () => {
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify({ token: 'jwt-here' }), {
				status: 200,
				headers: { 'Content-Type': 'application/json' }
			})
		);

		const result = await login('u@example.com', 'secret');

		expect(result).toEqual({ token: 'jwt-here' });
		expect(mockFetch).toHaveBeenCalledTimes(1);
		const [url, init] = mockFetch.mock.calls[0];
		expect(String(url)).toContain('/api/v1/auth/login');
		expect(init?.method).toBe('POST');
		expect(JSON.parse(init?.body as string)).toEqual({
			email: 'u@example.com',
			password: 'secret'
		});
	});

	it('login rejects with clear error on 401', async () => {
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(
				JSON.stringify({ error: 'unauthorized', message: 'Invalid email or password' }),
				{ status: 401, headers: { 'Content-Type': 'application/json' } }
			)
		);

		await expect(login('u@example.com', 'wrong')).rejects.toThrow('Invalid email or password');
	});

	it('when response has X-New-Token, setToken is called with that value', async () => {
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify({ token: 'old' }), {
				status: 200,
				headers: {
					'Content-Type': 'application/json',
					'X-New-Token': 'new-refreshed-token'
				}
			})
		);

		await login('u@example.com', 'secret');

		expect(auth.setToken).toHaveBeenCalledWith('new-refreshed-token');
	});
});
