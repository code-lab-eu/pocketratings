import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ProductList from '../../src/lib/ProductList.svelte';
import type { Product } from '../../src/lib/types';

const minimalProduct = (overrides: Partial<Product> = {}): Product => ({
  id: 'p1',
  category: { id: 'c1', name: 'Cat', ancestors: [] },
  brand: 'BrandCo',
  name: 'Widget',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  ...overrides
});

describe('ProductList', () => {
  it('uses hrefFor for product link hrefs', () => {
    const items = [{ product: minimalProduct() }];
    const hrefFor = (id: string) => `/p/${id}`;
    render(ProductList, {
      props: { items, hrefFor }
    });
    const link = screen.getByRole('link', { name: /widget/i });
    expect(link.getAttribute('href')).toBe('/p/p1');
  });

  it('renders empty list when items is empty', () => {
    render(ProductList, {
      props: { items: [], hrefFor: (id) => `/p/${id}` }
    });
    expect(screen.queryByRole('link')).not.toBeInTheDocument();
  });
});
