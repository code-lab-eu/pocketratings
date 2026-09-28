import { beforeEach, describe, expect, it, vi } from 'vitest';
import { load } from '../src/routes/products/[id]/+page';

const productId = '11111111-2222-4333-8444-555555555555';

/** Minimal LoadEvent-like object; load only uses params. */
function createLoadEvent(id: string): Parameters<typeof load>[0] {
  return {
    params: { id },
    url: new URL(`http://localhost/products/${id}`),
    fetch: vi.fn(),
    setHeaders: vi.fn(),
    parent: async () => ({}),
    depends: vi.fn(),
    data: null,
    untrack: vi.fn(),
    tracing: {},
    route: { id: null }
  } as unknown as Parameters<typeof load>[0];
}

const mocks = vi.hoisted(() => ({
  getProduct: vi.fn(),
  listReviews: vi.fn(),
  listPurchases: vi.fn(),
  listLocations: vi.fn()
}));

vi.mock('$lib/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('$lib/api')>();
  return {
    ...actual,
    getProduct: mocks.getProduct,
    listReviews: mocks.listReviews,
    listPurchases: mocks.listPurchases,
    listLocations: mocks.listLocations
  };
});

const product = { id: productId, name: 'Milk', variations: [] };
const locations = [{ id: 'loc-1', name: 'Store A' }];

describe('Product page load', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getProduct.mockResolvedValue(product);
    mocks.listReviews.mockResolvedValue([]);
    mocks.listPurchases.mockResolvedValue([]);
    mocks.listLocations.mockResolvedValue(locations);
  });

  it('loads locations for the inline add purchase form', async () => {
    const result = await load(createLoadEvent(productId));

    expect(mocks.listLocations).toHaveBeenCalledTimes(1);
    expect(result).toMatchObject({ product, locations, error: null, notFound: false });
  });

  it('returns the error and no locations when loading locations fails', async () => {
    mocks.listLocations.mockRejectedValue(new Error('Network down.'));

    const result = await load(createLoadEvent(productId));

    expect(result).toMatchObject({
      product: null,
      locations: [],
      notFound: false,
      error: 'Network down.'
    });
  });

  it('returns empty locations when the product id is invalid', async () => {
    const result = await load(createLoadEvent('not-a-uuid'));

    expect(mocks.listLocations).not.toHaveBeenCalled();
    expect(result).toMatchObject({ product: null, locations: [], notFound: true });
  });
});
