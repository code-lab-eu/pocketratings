import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ProductDetailPage from '../../src/routes/products/[id]/+page.svelte';
import type { PageData } from '../../src/routes/products/[id]/$types';
import type { ProductDetail, Purchase, Review } from '../../src/lib/types';

const product: ProductDetail = {
  id: 'prod-1',
  category: { id: 'cat-1', name: 'Dairy', ancestors: [] },
  brand: 'Acme',
  name: 'Milk',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  variations: []
};

const review: Review = {
  id: 'rev-1',
  product: { id: 'prod-1', brand: 'Acme', name: 'Milk' },
  user: { id: 'u1', name: 'Alice' },
  rating: 4,
  text: 'Good product',
  created_at: 1000,
  updated_at: 1000,
  deleted_at: null
};

const purchase: Purchase = {
  id: 'pur-1',
  user: { id: 'u1', name: 'Alice' },
  product: { id: 'prod-1', brand: 'Acme', name: 'Milk' },
  variation: { id: 'var-1', label: '', unit: 'none' },
  location: { id: 'loc-1', name: 'Store A' },
  quantity: 1,
  price: '2.99',
  purchased_at: 1708012800,
  deleted_at: null
};

const defaultData: PageData = {
  product,
  reviews: [review],
  purchases: [purchase],
  notFound: false,
  error: null
};

describe('Product detail page', () => {
  it('shows product name, brand, and category', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /milk/i })).toBeInTheDocument();
    expect(screen.getAllByText(/acme/i).length).toBeGreaterThanOrEqual(1);
    const dairyLinks = screen.getAllByRole('link', { name: /dairy/i });
    expect(dairyLinks).toHaveLength(2);
    dairyLinks.forEach((link) =>
      expect(link.getAttribute('href')).toContain('/categories/cat-1')
    );
  });

  it('shows breadcrumb with Home, category, and product name when product is loaded', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    expect(nav).toBeInTheDocument();
    const homeLink = screen.getByRole('link', { name: /^home$/i });
    expect(homeLink.getAttribute('href')).toContain('/');
    const categoryLinkInBreadcrumb = nav.querySelector('a[href*="/categories/cat-1"]');
    expect(categoryLinkInBreadcrumb).toBeInTheDocument();
    expect(categoryLinkInBreadcrumb).toHaveTextContent(/dairy/i);
    const currentSegment = nav.querySelector('[aria-current="page"]');
    expect(currentSegment).toBeInTheDocument();
    expect(currentSegment).toHaveTextContent(/acme - milk/i);
  });

  it('shows full breadcrumb with ancestors when product category has ancestors', () => {
    const productWithAncestors: ProductDetail = {
      ...product,
      category: {
        id: 'c2',
        name: 'Dairy',
        ancestors: [{ id: 'c1', name: 'Food' }]
      }
    };
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: productWithAncestors
        } as PageData
      }
    });
    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    const links = nav.querySelectorAll('a');
    expect(links).toHaveLength(3);
    expect(links[0]).toHaveAccessibleName('Home');
    expect(links[1]).toHaveAccessibleName('Food');
    expect(links[2]).toHaveAccessibleName('Dairy');
    const currentSegment = nav.querySelector('[aria-current="page"]');
    expect(currentSegment).toHaveTextContent(/acme - milk/i);
  });

  it('shows reviews section with rating, text, and user name', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /reviews/i })).toBeInTheDocument();
    expect(screen.getByText(/rating: 4\/5/i)).toBeInTheDocument();
    expect(screen.getByText(/good product/i)).toBeInTheDocument();
    expect(screen.getByText(/by alice/i)).toBeInTheDocument();
  });

  it('shows purchase history with date, location, variation, price', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    expect(screen.getByRole('heading', { name: /purchase history/i })).toBeInTheDocument();
    expect(screen.getByText(/store a/i)).toBeInTheDocument();
    expect(screen.getByText(/default/i)).toBeInTheDocument();
    expect(screen.getByText(/2\.99 €/)).toBeInTheDocument();
  });

  it('shows variation label in purchase history when set', () => {
    const purchaseWithLabel: Purchase = {
      ...purchase,
      variation: { id: 'var-2', label: '500 g', unit: 'grams' }
    };
    render(ProductDetailPage, {
      props: {
        data: { ...defaultData, purchases: [purchaseWithLabel] } as PageData
      }
    });
    expect(screen.getByText(/500 g/i)).toBeInTheDocument();
  });

  it('shows Add review and Add purchase placeholder links', () => {
    render(ProductDetailPage, {
      props: { data: defaultData }
    });
    const addReview = screen.getByRole('link', { name: /add review/i });
    const addPurchase = screen.getByRole('link', { name: /add purchase/i });
    expect(addReview).toBeInTheDocument();
    expect(addReview.getAttribute('href')).toContain('/manage/reviews/add');
    expect(addReview.getAttribute('href')).toContain('product_id=prod-1');
    expect(addPurchase).toBeInTheDocument();
    expect(addPurchase.getAttribute('href')).toContain('/manage/purchases/add');
    expect(addPurchase.getAttribute('href')).toContain('product_id=prod-1');
  });

  it('shows Not found and back link when notFound is true', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: true,
          error: null
        } as unknown as PageData
      }
    });
    expect(screen.getByText(/product not found/i)).toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Breadcrumb' })).not.toBeInTheDocument();
    const backLink = screen.getByRole('link', { name: /back to home/i });
    expect(backLink).toBeInTheDocument();
    expect(backLink.getAttribute('href')).toContain('/');
  });

  it('shows error when error is set', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: false,
          error: 'Not found'
        } as PageData
      }
    });
    expect(screen.getByText('Not found')).toBeInTheDocument();
    expect(screen.queryByRole('navigation', { name: 'Breadcrumb' })).not.toBeInTheDocument();
    const homeLink = screen.getByRole('link', { name: /home/i });
    expect(homeLink.getAttribute('href')).toContain('/');
  });

  it('shows product not found when product is null and no error and not notFound', () => {
    render(ProductDetailPage, {
      props: {
        data: {
          ...defaultData,
          product: null,
          reviews: [],
          purchases: [],
          notFound: false,
          error: null
        } as unknown as PageData
      }
    });
    expect(screen.getByText(/product not found/i)).toBeInTheDocument();
  });

  it('shows empty reviews message when no reviews', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, reviews: [] } as PageData }
    });
    expect(screen.getByText(/no reviews yet/i)).toBeInTheDocument();
  });

  it('shows empty purchase history message when no purchases', () => {
    render(ProductDetailPage, {
      props: { data: { ...defaultData, purchases: [] } as PageData }
    });
    expect(screen.getByText(/no purchases recorded/i)).toBeInTheDocument();
  });
});
