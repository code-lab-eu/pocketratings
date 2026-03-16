import { render } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';
import HomePage from '../../src/routes/+page.svelte';
import LoginPage from '../../src/routes/login/+page.svelte';
import ManagePage from '../../src/routes/manage/+page.svelte';
import CategoriesPage from '../../src/routes/manage/categories/+page.svelte';
import CategoryEditPage from '../../src/routes/manage/categories/[id]/+page.svelte';
import CategoryNewPage from '../../src/routes/manage/categories/new/+page.svelte';
import ProductsPage from '../../src/routes/manage/products/+page.svelte';
import ProductEditPage from '../../src/routes/manage/products/[id]/+page.svelte';
import ProductNewPage from '../../src/routes/manage/products/new/+page.svelte';
import ReviewsPage from '../../src/routes/manage/reviews/+page.svelte';
import ReviewAddPage from '../../src/routes/manage/reviews/add/+page.svelte';
import ReviewEditPage from '../../src/routes/manage/reviews/[id]/+page.svelte';
import PurchasesPage from '../../src/routes/manage/purchases/+page.svelte';
import PurchaseAddPage from '../../src/routes/manage/purchases/add/+page.svelte';
import PurchaseEditPage from '../../src/routes/manage/purchases/[id]/+page.svelte';
import LocationsPage from '../../src/routes/manage/locations/+page.svelte';
import LocationEditPage from '../../src/routes/manage/locations/[id]/+page.svelte';
import LocationNewPage from '../../src/routes/manage/locations/new/+page.svelte';
import type { PageData } from '../../src/routes/$types';
import type {
  Category,
  ProductDetail,
  Review,
  Location,
  Purchase
} from '../../src/lib/types';

const APP_SUFFIX = ' — Pocket Ratings';

afterEach(() => {
  document.title = '';
});

describe('Page titles', () => {
  it('home page sets title to Pocket Ratings', () => {
    render(HomePage, {
      props: {
        data: {
          categoriesTree: [],
          categories: [],
          items: [],
          query: '',
          error: null,
          fullCategories: []
        } as PageData
      }
    });
    expect(document.title).toBe('Pocket Ratings');
  });

  it('login page sets title', () => {
    render(LoginPage);
    expect(document.title).toBe('Login' + APP_SUFFIX);
  });

  it('manage hub sets title', () => {
    render(ManagePage);
    expect(document.title).toBe('Manage' + APP_SUFFIX);
  });

  it('manage categories list sets title', () => {
    render(CategoriesPage, {
      props: { data: { categories: [], error: null } }
    });
    expect(document.title).toBe('Categories' + APP_SUFFIX);
  });

  it('manage category edit sets title from category name', () => {
    const category: Category = {
      id: 'c1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryEditPage, {
      props: {
        data: {
          category,
          categories: [category],
          error: null,
          notFound: false
        }
      }
    });
    expect(document.title).toBe('Edit category: Food' + APP_SUFFIX);
  });

  it('manage category edit sets fallback title when not found', () => {
    render(CategoryEditPage, {
      props: {
        data: {
          category: null,
          categories: [],
          error: null,
          notFound: true
        }
      }
    });
    expect(document.title).toBe('Category' + APP_SUFFIX);
  });

  it('manage new category sets title', () => {
    render(CategoryNewPage, {
      props: { data: { categories: [], error: null } }
    });
    expect(document.title).toBe('New category' + APP_SUFFIX);
  });

  it('manage products list sets title', () => {
    render(ProductsPage, {
      props: { data: { products: [], error: null } }
    });
    expect(document.title).toBe('Products' + APP_SUFFIX);
  });

  it('manage product edit sets title from product name', () => {
    const product: ProductDetail = {
      id: 'p1',
      category: { id: 'c1', name: 'Food', ancestors: [] },
      brand: 'Acme',
      name: 'Milk',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      review_score: 4,
      price: '2.99',
      variations: []
    };
    render(ProductEditPage, {
      props: {
        data: {
          product,
          categories: [],
          error: null,
          notFound: false
        }
      }
    });
    expect(document.title).toBe('Edit product: Acme - Milk' + APP_SUFFIX);
  });

  it('manage new product sets title', () => {
    render(ProductNewPage, {
      props: { data: { categories: [], categoryId: null, error: null } }
    });
    expect(document.title).toBe('New product' + APP_SUFFIX);
  });

  it('manage reviews list sets title', () => {
    render(ReviewsPage, {
      props: { data: { reviews: [], error: null } }
    });
    expect(document.title).toBe('Reviews' + APP_SUFFIX);
  });

  it('manage add review sets title', () => {
    render(ReviewAddPage, {
      props: { data: { products: [], productId: undefined, error: null } }
    });
    expect(document.title).toBe('Add review' + APP_SUFFIX);
  });

  it('manage edit review sets title', () => {
    const review: Review = {
      id: 'r1',
      product: { id: 'p1', brand: 'Acme', name: 'Milk' },
      user: { id: 'u1', name: 'User' },
      rating: 4,
      text: null,
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(ReviewEditPage, {
      props: {
        data: {
          review,
          error: null
        }
      }
    });
    expect(document.title).toBe('Edit review' + APP_SUFFIX);
  });

  it('manage purchases list sets title', () => {
    render(PurchasesPage, {
      props: { data: { purchases: [], error: null } }
    });
    expect(document.title).toBe('Purchases' + APP_SUFFIX);
  });

  it('manage add purchase sets title', () => {
    render(PurchaseAddPage, {
      props: {
        data: {
          products: [],
          locations: [],
          productId: undefined,
          error: null
        }
      }
    });
    expect(document.title).toBe('Add purchase' + APP_SUFFIX);
  });

  it('manage edit purchase sets title', () => {
    const purchase: Purchase = {
      id: 'pur1',
      user: { id: 'u1', name: 'User' },
      product: { id: 'p1', brand: 'Acme', name: 'Milk' },
      variation: { id: 'v1', label: '1 L', unit: 'volume', quantity: 1 },
      location: { id: 'loc1', name: 'Store' },
      quantity: 1,
      price: '2.99',
      purchased_at: 0,
      deleted_at: null
    };
    render(PurchaseEditPage, {
      props: {
        data: {
          purchase,
          products: [],
          locations: [],
          error: null
        }
      }
    });
    expect(document.title).toBe('Edit purchase' + APP_SUFFIX);
  });

  it('manage locations list sets title', () => {
    render(LocationsPage, {
      props: { data: { locations: [], error: null } }
    });
    expect(document.title).toBe('Locations' + APP_SUFFIX);
  });

  it('manage location edit sets title from location name', () => {
    const location: Location = {
      id: 'loc1',
      name: 'Supermarket',
      deleted_at: null
    };
    render(LocationEditPage, {
      props: {
        data: {
          location,
          error: null,
          notFound: false
        }
      }
    });
    expect(document.title).toBe('Edit location: Supermarket' + APP_SUFFIX);
  });

  it('manage new location sets title', () => {
    render(LocationNewPage);
    expect(document.title).toBe('New location' + APP_SUFFIX);
  });
});
