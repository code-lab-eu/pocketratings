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
  getProductVariations,
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

  function mockAuth() {
    vi.mocked(auth.getToken).mockReturnValue('t');
  }

  function mockJsonResponse<T>(data: T, status = 200) {
    vi.mocked(fetch).mockResolvedValueOnce(
      new Response(JSON.stringify(data), {
        status,
        headers: { 'Content-Type': 'application/json' }
      })
    );
  }

  function mockEmptyResponse(status = 200) {
    vi.mocked(fetch).mockResolvedValueOnce(new Response('', { status }));
  }

  function categoryFixture(overrides?: Record<string, unknown>) {
    return {
      id: 'c1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      ...overrides
    };
  }

  function productFixture(overrides?: Record<string, unknown>) {
    return {
      id: 'p1',
      category: { id: 'c1', name: 'Category', ancestors: [] },
      brand: 'Brand',
      name: 'Product',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      ...overrides
    };
  }

  function purchaseFixture(overrides?: Record<string, unknown>) {
    return {
      id: 'pur1',
      user: { id: 'u1', name: 'Alice' },
      product: { id: 'p1', brand: 'Brand', name: 'Product' },
      location: { id: 'loc1', name: 'Store' },
      quantity: 1,
      price: '2.99',
      purchased_at: 1708012800,
      deleted_at: null,
      ...overrides
    };
  }

  function reviewFixture(overrides?: Record<string, unknown>) {
    return {
      id: 'r1',
      product: { id: 'p1', brand: 'B', name: 'Product' },
      user: { id: 'u1', name: 'User' },
      rating: 4,
      text: 'Good',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      ...overrides
    };
  }

  function locationFixture(overrides?: Record<string, unknown>) {
    return { id: 'loc1', name: 'Store A', deleted_at: null, ...overrides };
  }

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
    const categories = [categoryFixture()];
    mockJsonResponse(categories);
    const mockFetch = vi.mocked(fetch);

    const result = await listCategories();

    expect(result).toEqual(categories);
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories');
    expect(String(mockFetch.mock.calls[0][0])).not.toContain('parent_id');
  });

  it('listCategories with parentId adds query param', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listCategories('parent-uuid');

    expect(String(mockFetch.mock.calls[0][0])).toContain('parent_id=parent-uuid');
  });

  it('getCategory fetches GET /api/v1/categories/:id', async () => {
    mockAuth();
    const cat = categoryFixture({ id: 'cid', name: 'Drinks' });
    mockJsonResponse(cat);
    const mockFetch = vi.mocked(fetch);

    const result = await getCategory('cid');

    expect(result).toEqual(cat);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/cid');
    expect(String(mockFetch.mock.calls[0][0])).not.toContain('depth=');
  });

  it('getCategory with depth adds query param', async () => {
    mockAuth();
    mockJsonResponse(categoryFixture({ id: 'c', name: 'C' }));
    const mockFetch = vi.mocked(fetch);

    await getCategory('c', { depth: 2 });

    expect(String(mockFetch.mock.calls[0][0])).toContain('depth=2');
  });

  it('listProducts with category_id fetches with query param', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listProducts({ category_id: 'cat-1' });

    expect(String(mockFetch.mock.calls[0][0])).toContain('category_id=cat-1');
  });

  it('listProducts with q fetches with query param', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listProducts({ q: 'milk' });

    expect(String(mockFetch.mock.calls[0][0])).toContain('q=milk');
  });

  it('listReviews without productId fetches GET /api/v1/reviews', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listReviews();

    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/reviews');
    expect(String(mockFetch.mock.calls[0][0])).not.toContain('product_id');
  });

  it('listReviews with productId adds query param', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listReviews('prod-1');

    expect(String(mockFetch.mock.calls[0][0])).toContain('product_id=prod-1');
  });

  it('getProduct fetches GET /api/v1/products/:id and returns product', async () => {
    mockAuth();
    const product = productFixture({ id: 'pid', category: { id: 'cid', name: 'Category', ancestors: [] } });
    mockJsonResponse(product);
    const mockFetch = vi.mocked(fetch);

    const result = await getProduct('pid');

    expect(result).toEqual(product);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/products/pid');
  });

  it('listPurchases without options fetches GET /api/v1/purchases', async () => {
    mockAuth();
    mockJsonResponse([]);
    const mockFetch = vi.mocked(fetch);

    await listPurchases();

    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/purchases');
    expect(String(mockFetch.mock.calls[0][0])).not.toContain('product_id');
  });

  it('listPurchases with product_id adds query param', async () => {
    mockAuth();
    const purchases = [
      purchaseFixture({
        id: 'p1',
        product: { id: 'prod-1', brand: 'Brand', name: 'Product' },
        location: { id: 'loc-1', name: 'Store' }
      })
    ];
    mockJsonResponse(purchases);
    const mockFetch = vi.mocked(fetch);

    const result = await listPurchases({ product_id: 'prod-1' });

    expect(result).toEqual(purchases);
    expect(String(mockFetch.mock.calls[0][0])).toContain('product_id=prod-1');
  });

  it('listLocations fetches GET /api/v1/locations and returns array', async () => {
    mockAuth();
    const locations = [locationFixture()];
    mockJsonResponse(locations);
    const mockFetch = vi.mocked(fetch);

    const result = await listLocations();

    expect(result).toEqual(locations);
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations');
  });

  it('createCategory sends POST to /api/v1/categories and returns category', async () => {
    mockAuth();
    const created = categoryFixture();
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    const result = await createCategory({ name: 'Food' });

    expect(result).toEqual(created);
    expect(initMethod(mockFetch)).toBe('POST');
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories');
    expect(JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string)).toEqual({ name: 'Food' });
  });

  it('updateCategory sends PATCH to /api/v1/categories/:id', async () => {
    mockAuth();
    const updated = categoryFixture({ name: 'Food (renamed)' });
    mockJsonResponse(updated);
    const mockFetch = vi.mocked(fetch);

    const result = await updateCategory('c1', { name: 'Food (renamed)' });

    expect(result.name).toBe('Food (renamed)');
    expect(initMethod(mockFetch)).toBe('PATCH');
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/c1');
  });

  it('deleteCategory sends DELETE to /api/v1/categories/:id', async () => {
    mockAuth();
    mockEmptyResponse();
    const mockFetch = vi.mocked(fetch);

    await deleteCategory('c1');

    expect(initMethod(mockFetch)).toBe('DELETE');
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/categories/c1');
  });

  it('getLocation fetches GET /api/v1/locations/:id', async () => {
    mockAuth();
    const loc = locationFixture();
    mockJsonResponse(loc);
    const mockFetch = vi.mocked(fetch);

    const result = await getLocation('loc1');

    expect(result).toEqual(loc);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations/loc1');
  });

  it('createLocation sends POST to /api/v1/locations', async () => {
    mockAuth();
    const created = locationFixture();
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    const result = await createLocation({ name: 'Store A' });

    expect(result).toEqual(created);
    expect(initMethod(mockFetch)).toBe('POST');
    expect(JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string)).toEqual({ name: 'Store A' });
  });

  it('updateLocation sends PATCH to /api/v1/locations/:id', async () => {
    mockAuth();
    mockJsonResponse(locationFixture({ name: 'Store B' }));
    const mockFetch = vi.mocked(fetch);

    await updateLocation('loc1', { name: 'Store B' });

    expect(initMethod(mockFetch)).toBe('PATCH');
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/locations/loc1');
  });

  it('deleteLocation sends DELETE to /api/v1/locations/:id', async () => {
    mockAuth();
    mockEmptyResponse();
    const mockFetch = vi.mocked(fetch);

    await deleteLocation('loc1');

    expect(initMethod(mockFetch)).toBe('DELETE');
  });

  it('createProduct sends POST to /api/v1/products', async () => {
    mockAuth();
    const created = productFixture({
      category: { id: 'c1', name: 'Groceries', ancestors: [] },
      brand: 'B',
      name: 'Milk'
    });
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    const result = await createProduct({ name: 'Milk', brand: 'B', category_id: 'c1' });

    expect(result).toEqual(created);
    expect(initMethod(mockFetch)).toBe('POST');
  });

  it('createProduct with first_variation sends it in the body', async () => {
    mockAuth();
    const created = productFixture({
      id: 'p2',
      category: { id: 'c1', name: 'Groceries', ancestors: [] },
      brand: 'Dairy',
      name: 'Milk 1L'
    });
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    await createProduct({
      name: 'Milk 1L',
      brand: 'Dairy',
      category_id: 'c1',
      first_variation: { label: '1 L', unit: 'milliliters', quantity: 1000 }
    });

    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          name: 'Milk 1L',
          brand: 'Dairy',
          category_id: 'c1',
          first_variation: { label: '1 L', unit: 'milliliters', quantity: 1000 }
        })
      })
    );
  });

  it('updateProduct sends PATCH to /api/v1/products/:id', async () => {
    mockAuth();
    mockJsonResponse(
      productFixture({
        category: { id: 'c1', name: 'Groceries', ancestors: [] },
        brand: 'B',
        name: 'Milk 1L'
      })
    );
    const mockFetch = vi.mocked(fetch);

    await updateProduct('p1', { name: 'Milk 1L' });

    expect(initMethod(mockFetch)).toBe('PATCH');
  });

  it('deleteProduct sends DELETE to /api/v1/products/:id', async () => {
    mockAuth();
    mockEmptyResponse();
    const mockFetch = vi.mocked(fetch);

    await deleteProduct('p1');

    expect(initMethod(mockFetch)).toBe('DELETE');
  });

  it('getProductVariations fetches GET /api/v1/products/:id/variations and returns array', async () => {
    mockAuth();
    const variations = [
      { id: 'v1', label: '500 g', unit: 'g' },
      { id: 'v2', label: '', unit: 'none' }
    ];
    mockJsonResponse(variations);
    const mockFetch = vi.mocked(fetch);

    const result = await getProductVariations('prod-1');

    expect(result).toEqual(variations);
    expect(mockFetch).toHaveBeenCalledTimes(1);
    expect(String(mockFetch.mock.calls[0][0])).toContain(
      '/api/v1/products/prod-1/variations'
    );
  });

  it('getPurchase fetches GET /api/v1/purchases/:id', async () => {
    mockAuth();
    const purchase = purchaseFixture();
    mockJsonResponse(purchase);
    const mockFetch = vi.mocked(fetch);

    const result = await getPurchase('pur1');

    expect(result).toEqual(purchase);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/purchases/pur1');
  });

  it('createPurchase sends POST to /api/v1/purchases', async () => {
    mockAuth();
    const created = purchaseFixture();
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    const result = await createPurchase({ product_id: 'p1', location_id: 'loc1', price: '2.99' });

    expect(result).toEqual(created);
    expect(initMethod(mockFetch)).toBe('POST');
  });

  it('createPurchase sends variation_id in body when provided', async () => {
    mockAuth();
    const created = purchaseFixture({
      variation: { id: 'v1', label: '500 g', unit: 'g' }
    });
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    await createPurchase({
      product_id: 'p1',
      variation_id: 'v1',
      location_id: 'loc1',
      price: '2.99'
    });

    const body = JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string);
    expect(body.variation_id).toBe('v1');
  });

  it('updatePurchase sends PATCH to /api/v1/purchases/:id', async () => {
    mockAuth();
    mockJsonResponse(purchaseFixture({ quantity: 2, price: '3.49' }));
    const mockFetch = vi.mocked(fetch);

    await updatePurchase('pur1', { quantity: 2, price: '3.49' });

    expect(initMethod(mockFetch)).toBe('PATCH');
  });

  it('updatePurchase sends variation_id in body when provided', async () => {
    mockAuth();
    mockJsonResponse(purchaseFixture());
    const mockFetch = vi.mocked(fetch);

    await updatePurchase('pur1', { variation_id: 'v2' });

    const body = JSON.parse((mockFetch.mock.calls[0][1] as RequestInit).body as string);
    expect(body.variation_id).toBe('v2');
  });

  it('deletePurchase sends DELETE to /api/v1/purchases/:id', async () => {
    mockAuth();
    mockEmptyResponse();
    const mockFetch = vi.mocked(fetch);

    await deletePurchase('pur1');

    expect(initMethod(mockFetch)).toBe('DELETE');
  });

  it('getReview fetches GET /api/v1/reviews/:id', async () => {
    mockAuth();
    const review = reviewFixture();
    mockJsonResponse(review);
    const mockFetch = vi.mocked(fetch);

    const result = await getReview('r1');

    expect(result).toEqual(review);
    expect(String(mockFetch.mock.calls[0][0])).toContain('/api/v1/reviews/r1');
  });

  it('createReview sends POST to /api/v1/reviews', async () => {
    mockAuth();
    const created = reviewFixture();
    mockJsonResponse(created, 201);
    const mockFetch = vi.mocked(fetch);

    const result = await createReview({ product_id: 'p1', rating: 4, text: 'Good' });

    expect(result).toEqual(created);
    expect(initMethod(mockFetch)).toBe('POST');
  });

  it('updateReview sends PATCH to /api/v1/reviews/:id', async () => {
    mockAuth();
    mockJsonResponse(reviewFixture({ rating: 5, text: 'Excellent' }));
    const mockFetch = vi.mocked(fetch);

    await updateReview('r1', { rating: 5, text: 'Excellent' });

    expect(initMethod(mockFetch)).toBe('PATCH');
  });

  it('deleteReview sends DELETE to /api/v1/reviews/:id', async () => {
    mockAuth();
    mockEmptyResponse();
    const mockFetch = vi.mocked(fetch);

    await deleteReview('r1');

    expect(initMethod(mockFetch)).toBe('DELETE');
  });
});

function initMethod(mockFetch: ReturnType<typeof vi.mocked<typeof fetch>>): string {
  return (mockFetch.mock.calls[0][1] as RequestInit)?.method ?? '';
}
