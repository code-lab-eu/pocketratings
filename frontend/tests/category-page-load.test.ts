import { beforeEach, describe, expect, it, vi } from 'vitest';
import { load } from '../src/routes/categories/[id]/+page';

const categoryId = '11111111-2222-4333-8444-555555555555';

/** Minimal LoadEvent-like object; load only uses params and url. */
function createLoadEvent(overrides: { params: { id: string }; url: URL }): Parameters<typeof load>[0] {
  return {
    params: overrides.params,
    url: overrides.url,
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
  getCategory: vi.fn(),
  listProducts: vi.fn(),
  isValidUuid: vi.fn()
}));

vi.mock('$lib/api', () => ({
  ApiClientError: class ApiClientError extends Error {
    constructor(message: string, public status: number, public errorCode: string) {
      super(message);
      this.name = 'ApiClientError';
    }
  },
  isValidUuid: (v: string) => mocks.isValidUuid(v),
  getCategory: (...args: unknown[]) => mocks.getCategory(...args),
  listProducts: (...args: unknown[]) => mocks.listProducts(...args)
}));

describe('Category page load', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('calls listProducts with category_id from params', async () => {
    mocks.isValidUuid.mockReturnValue(true);
    mocks.getCategory.mockResolvedValue({
      id: categoryId,
      name: 'Food',
      ancestors: [],
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    });
    mocks.listProducts.mockResolvedValue([]);

    await load(
      createLoadEvent({
        params: { id: categoryId },
        url: new URL('https://app.example/categories/' + categoryId)
      })
    );

    expect(mocks.listProducts).toHaveBeenCalledTimes(1);
    expect(mocks.listProducts).toHaveBeenCalledWith({ category_id: categoryId });
  });

  it('calls listProducts with category_id and q when search query is present', async () => {
    mocks.isValidUuid.mockReturnValue(true);
    mocks.getCategory.mockResolvedValue({
      id: categoryId,
      name: 'Food',
      ancestors: [],
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    });
    mocks.listProducts.mockResolvedValue([]);

    await load(
      createLoadEvent({
        params: { id: categoryId },
        url: new URL('https://app.example/categories/' + categoryId + '?q=milk')
      })
    );

    expect(mocks.listProducts).toHaveBeenCalledTimes(1);
    expect(mocks.listProducts).toHaveBeenCalledWith({
      category_id: categoryId,
      q: 'milk'
    });
  });

  it('calls getCategory with depth 2 to load two levels of children for expandable list', async () => {
    mocks.isValidUuid.mockReturnValue(true);
    mocks.getCategory.mockResolvedValue({
      id: categoryId,
      name: 'Food',
      ancestors: [],
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      children: []
    });
    mocks.listProducts.mockResolvedValue([]);

    await load(
      createLoadEvent({
        params: { id: categoryId },
        url: new URL('https://app.example/categories/' + categoryId)
      })
    );

    expect(mocks.getCategory).toHaveBeenCalledTimes(1);
    expect(mocks.getCategory).toHaveBeenCalledWith(categoryId, { depth: 2 });
  });
});
