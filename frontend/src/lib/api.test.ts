import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as auth from './auth';
import {
	createCategory,
	createLocation,
	createProduct,
	createPurchase,
	createReview,
	deleteCategory,
	deleteLocation,
	deleteProduct,
	deletePurchase,
	deleteReview,
	getCategory,
	getLocation,
	getProduct,
	getPurchase,
	getReview,
	listCategories,
	listLocations,
	listProducts,
	listPurchases,
	listReviews,
	login,
	updateCategory,
	updateLocation,
	updateProduct,
	updatePurchase,
	updateReview
} from './api';

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

	it('listCategories fetches GET /api/v1/categories and returns array', async () => {
		vi.mocked(auth.getToken).mockReturnValue('token');
		const mockFetch = vi.mocked(fetch);
		const categories = [
			{ id: 'c1', parent_id: null, name: 'Food', created_at: 0, updated_at: 0, deleted_at: null }
		];
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(categories), {
				status: 200,
				headers: { 'Content-Type': 'application/json' }
			})
		);

		const result = await listCategories();

		expect(result).toEqual(categories);
		expect(mockFetch).toHaveBeenCalledTimes(1);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories');
		expect(String(mockFetch.mock.calls[0][0])).not.toContain('parent_id');
	});

	it('listCategories with parentId adds query param', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listCategories('parent-uuid');

		expect(String(mockFetch.mock.calls[0][0])).toContain('parent_id=parent-uuid');
	});

	it('getCategory fetches GET /api/v1/categories/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const cat = {
			id: 'cid',
			parent_id: null,
			name: 'Drinks',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(cat), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await getCategory('cid');

		expect(result).toEqual(cat);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/cid');
	});

	it('listProducts with category_id fetches with query param', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listProducts({ category_id: 'cat-1' });

		expect(String(mockFetch.mock.calls[0][0])).toContain('category_id=cat-1');
	});

	it('listProducts with q fetches with query param', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listProducts({ q: 'milk' });

		expect(String(mockFetch.mock.calls[0][0])).toContain('q=milk');
	});

	it('listReviews without productId fetches GET /api/v1/reviews', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listReviews();

		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/reviews');
		expect(String(mockFetch.mock.calls[0][0])).not.toContain('product_id');
	});

	it('listReviews with productId adds query param', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listReviews('prod-1');

		expect(String(mockFetch.mock.calls[0][0])).toContain('product_id=prod-1');
	});

	it('getProduct fetches GET /api/v1/products/:id and returns product', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const product = {
			id: 'pid',
			category_id: 'cid',
			brand: 'Brand',
			name: 'Product',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(product), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await getProduct('pid');

		expect(result).toEqual(product);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/products/pid');
	});

	it('listPurchases without options fetches GET /api/v1/purchases', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify([]), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		await listPurchases();

		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/purchases');
		expect(String(mockFetch.mock.calls[0][0])).not.toContain('product_id');
	});

	it('listPurchases with product_id adds query param', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const purchases = [
			{
				id: 'p1',
				user: { id: 'u1', name: 'Alice' },
				product: { id: 'prod-1', brand: 'Brand', name: 'Product' },
				location: { id: 'loc-1', name: 'Store' },
				quantity: 1,
				price: '2.99',
				purchased_at: 1708012800,
				deleted_at: null
			}
		];
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(purchases), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await listPurchases({ product_id: 'prod-1' });

		expect(result).toEqual(purchases);
		expect(String(mockFetch.mock.calls[0][0])).toContain('product_id=prod-1');
	});

	it('listLocations fetches GET /api/v1/locations and returns array', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const locations = [{ id: 'loc1', name: 'Store A', deleted_at: null }];
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(locations), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await listLocations();

		expect(result).toEqual(locations);
		expect(mockFetch).toHaveBeenCalledTimes(1);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations');
	});

	it('createCategory sends POST to /api/v1/categories and returns category', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const created = {
			id: 'c1',
			parent_id: null,
			name: 'Food',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(created), { status: 201, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await createCategory({ name: 'Food' });

		expect(result).toEqual(created);
		expect(initMethod(mockFetch)).toBe('POST');
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories');
		expect(JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string)).toEqual({ name: 'Food' });
	});

	it('updateCategory sends PATCH to /api/v1/categories/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const updated = {
			id: 'c1',
			parent_id: null,
			name: 'Food (renamed)',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(updated), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await updateCategory('c1', { name: 'Food (renamed)' });

		expect(result.name).toBe('Food (renamed)');
		expect(initMethod(mockFetch)).toBe('PATCH');
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/c1');
	});

	it('deleteCategory sends DELETE to /api/v1/categories/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(new Response('', { status: 200 }));

		await deleteCategory('c1');

		expect(initMethod(mockFetch)).toBe('DELETE');
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/c1');
	});

	it('getLocation fetches GET /api/v1/locations/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const loc = { id: 'loc1', name: 'Store A', deleted_at: null };
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(loc), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await getLocation('loc1');

		expect(result).toEqual(loc);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations/loc1');
	});

	it('createLocation sends POST to /api/v1/locations', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const created = { id: 'loc1', name: 'Store A', deleted_at: null };
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(created), { status: 201, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await createLocation({ name: 'Store A' });

		expect(result).toEqual(created);
		expect(initMethod(mockFetch)).toBe('POST');
		expect(JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string)).toEqual({ name: 'Store A' });
	});

	it('updateLocation sends PATCH to /api/v1/locations/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify({ id: 'loc1', name: 'Store B', deleted_at: null }), {
				status: 200,
				headers: { 'Content-Type': 'application/json' }
			})
		);

		await updateLocation('loc1', { name: 'Store B' });

		expect(initMethod(mockFetch)).toBe('PATCH');
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations/loc1');
	});

	it('deleteLocation sends DELETE to /api/v1/locations/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(new Response('', { status: 200 }));

		await deleteLocation('loc1');

		expect(initMethod(mockFetch)).toBe('DELETE');
	});

	it('createProduct sends POST to /api/v1/products', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const created = {
			id: 'p1',
			category_id: 'c1',
			brand: 'B',
			name: 'Milk',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(created), { status: 201, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await createProduct({ name: 'Milk', brand: 'B', category_id: 'c1' });

		expect(result).toEqual(created);
		expect(initMethod(mockFetch)).toBe('POST');
	});

	it('updateProduct sends PATCH to /api/v1/products/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(
				JSON.stringify({
					id: 'p1',
					category_id: 'c1',
					brand: 'B',
					name: 'Milk 1L',
					created_at: 0,
					updated_at: 0,
					deleted_at: null
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		);

		await updateProduct('p1', { name: 'Milk 1L' });

		expect(initMethod(mockFetch)).toBe('PATCH');
	});

	it('deleteProduct sends DELETE to /api/v1/products/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(new Response('', { status: 200 }));

		await deleteProduct('p1');

		expect(initMethod(mockFetch)).toBe('DELETE');
	});

	it('getPurchase fetches GET /api/v1/purchases/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const purchase = {
			id: 'pur1',
			user: { id: 'u1', name: 'Alice' },
			product: { id: 'p1', brand: 'Brand', name: 'Product' },
			location: { id: 'loc1', name: 'Store' },
			quantity: 1,
			price: '2.99',
			purchased_at: 1708012800,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(purchase), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await getPurchase('pur1');

		expect(result).toEqual(purchase);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/purchases/pur1');
	});

	it('createPurchase sends POST to /api/v1/purchases', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const created = {
			id: 'pur1',
			user: { id: 'u1', name: 'Alice' },
			product: { id: 'p1', brand: 'Brand', name: 'Product' },
			location: { id: 'loc1', name: 'Store' },
			quantity: 1,
			price: '2.99',
			purchased_at: 1708012800,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(created), { status: 201, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await createPurchase({ product_id: 'p1', location_id: 'loc1', price: '2.99' });

		expect(result).toEqual(created);
		expect(initMethod(mockFetch)).toBe('POST');
	});

	it('updatePurchase sends PATCH to /api/v1/purchases/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(
				JSON.stringify({
					id: 'pur1',
					user: { id: 'u1', name: 'Alice' },
					product: { id: 'p1', brand: 'Brand', name: 'Product' },
					location: { id: 'loc1', name: 'Store' },
					quantity: 2,
					price: '3.49',
					purchased_at: 1708012800,
					deleted_at: null
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		);

		await updatePurchase('pur1', { quantity: 2, price: '3.49' });

		expect(initMethod(mockFetch)).toBe('PATCH');
	});

	it('deletePurchase sends DELETE to /api/v1/purchases/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(new Response('', { status: 200 }));

		await deletePurchase('pur1');

		expect(initMethod(mockFetch)).toBe('DELETE');
	});

	it('getReview fetches GET /api/v1/reviews/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const review = {
			id: 'r1',
			product_id: 'p1',
			user_id: 'u1',
			rating: 4,
			text: 'Good',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(review), { status: 200, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await getReview('r1');

		expect(result).toEqual(review);
		expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/reviews/r1');
	});

	it('createReview sends POST to /api/v1/reviews', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		const created = {
			id: 'r1',
			product_id: 'p1',
			user_id: 'u1',
			rating: 4,
			text: 'Good',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		mockFetch.mockResolvedValueOnce(
			new Response(JSON.stringify(created), { status: 201, headers: { 'Content-Type': 'application/json' } })
		);

		const result = await createReview({ product_id: 'p1', rating: 4, text: 'Good' });

		expect(result).toEqual(created);
		expect(initMethod(mockFetch)).toBe('POST');
	});

	it('updateReview sends PATCH to /api/v1/reviews/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(
			new Response(
				JSON.stringify({
					id: 'r1',
					product_id: 'p1',
					user_id: 'u1',
					rating: 5,
					text: 'Excellent',
					created_at: 0,
					updated_at: 0,
					deleted_at: null
				}),
				{ status: 200, headers: { 'Content-Type': 'application/json' } }
			)
		);

		await updateReview('r1', { rating: 5, text: 'Excellent' });

		expect(initMethod(mockFetch)).toBe('PATCH');
	});

	it('deleteReview sends DELETE to /api/v1/reviews/:id', async () => {
		vi.mocked(auth.getToken).mockReturnValue('t');
		const mockFetch = vi.mocked(fetch);
		mockFetch.mockResolvedValueOnce(new Response('', { status: 200 }));

		await deleteReview('r1');

		expect(initMethod(mockFetch)).toBe('DELETE');
	});
});

function initMethod(mockFetch: ReturnType<typeof vi.mocked<typeof fetch>>): string {
	return (mockFetch.mock.calls[0][1] as RequestInit)?.method ?? '';
}
