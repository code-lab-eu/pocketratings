import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import HomePage from '../../src/routes/+page.svelte';
import type { PageData } from '../../src/routes/$types';
import type { Category, Product } from '../../src/lib/types';

const dairy: Category = {
  id: 'cat-2',
  ancestors: [{ id: 'cat-1', name: 'Food' }],
  name: 'Dairy',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: []
};

const food: Category = {
  id: 'cat-1',
  ancestors: [],
  name: 'Food',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: [dairy]
};

const drinks: Category = {
  id: 'cat-3',
  ancestors: [],
  name: 'Drinks',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: []
};

const defaultData: PageData = {
  categoriesTree: [],
  categories: [],
  items: [],
  query: '',
  error: null,
  fullCategories: []
};

describe('Home page', () => {
  it('renders search bar with placeholder', () => {
    render(HomePage, {
      props: { data: { ...defaultData } }
    });
    expect(screen.getByLabelText(/search categories and products/i)).toBeInTheDocument();
    expect(screen.getByPlaceholderText(/search categories and products/i)).toBeInTheDocument();
  });

  it('form action is home (submit goes to /?q=...)', () => {
    render(HomePage, {
      props: { data: { ...defaultData } }
    });
    const form = screen.getByRole('search').closest('form');
    expect(form).toHaveAttribute('action');
    expect(form?.getAttribute('action')).toMatch(/\/$/);
  });

  it('shows Categories and Products sections', () => {
    render(HomePage, {
      props: { data: { ...defaultData } }
    });
    expect(screen.getByRole('heading', { name: /categories/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: /products/i })).toBeInTheDocument();
  });

  it('shows empty categories message when no categories', () => {
    render(HomePage, {
      props: { data: { ...defaultData } }
    });
    expect(screen.getByText(/no categories match/i)).toBeInTheDocument();
  });

  it('shows empty products message when no items', () => {
    render(HomePage, {
      props: { data: { ...defaultData } }
    });
    expect(screen.getByText(/no products match/i)).toBeInTheDocument();
  });

  it('shows only root categories initially (not children)', () => {
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          categoriesTree: [food, drinks]
        } as PageData
      }
    });
    expect(screen.getByRole('link', { name: /food/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /drinks/i })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /dairy/i })).not.toBeInTheDocument();
  });

  it('shows expand button for category with children', () => {
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          categoriesTree: [food, drinks]
        } as PageData
      }
    });
    expect(screen.getByRole('button', { name: /expand food/i })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /expand drinks/i })).not.toBeInTheDocument();
  });

  it('expands category to show children when expand button clicked', async () => {
    const user = userEvent.setup();
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          categoriesTree: [food, drinks]
        } as PageData
      }
    });
    expect(screen.queryByRole('link', { name: /dairy/i })).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /expand food/i }));
    expect(screen.getByRole('link', { name: /dairy/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /collapse food/i })).toBeInTheDocument();
  });

  it('collapses category when collapse button clicked', async () => {
    const user = userEvent.setup();
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          categoriesTree: [food, drinks]
        } as PageData
      }
    });
    await user.click(screen.getByRole('button', { name: /expand food/i }));
    expect(screen.getByRole('link', { name: /dairy/i })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /collapse food/i }));
    expect(screen.queryByRole('link', { name: /dairy/i })).not.toBeInTheDocument();
  });

  it('shows product list when items are provided', () => {
    const product: Product = {
      id: 'prod-1',
      category: { id: 'cat-1', name: 'Groceries', ancestors: [] },
      brand: 'Acme',
      name: 'Milk',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      review_score: 4,
      price: '2.99'
    };
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          items: [{ product }]
        } as PageData
      }
    });
    const link = screen.getByRole('link', { name: /milk/i });
    expect(link).toBeInTheDocument();
    expect(link.getAttribute('href')).toContain('/products/prod-1');
    expect(screen.getByLabelText(/rating: 4\.0 out of 5/i)).toBeInTheDocument();
    expect(screen.getByText('2.99 €')).toBeInTheDocument();
  });

  it('shows error message when error is set (no categories or products)', () => {
    render(HomePage, {
      props: { data: { ...defaultData, error: 'Network error' } as PageData }
    });
    expect(screen.getByText('Network error')).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: /categories/i })).not.toBeInTheDocument();
  });

  it('reflects query in search input value', () => {
    render(HomePage, {
      props: { data: { ...defaultData, query: 'milk' } as PageData }
    });
    const input = screen.getByRole('searchbox');
    expect(input).toHaveValue('milk');
  });

  it('shows flat filtered categories during search (no expand controls)', () => {
    const flatCategories = [
      { category: food, depth: 0 },
      { category: dairy, depth: 1 }
    ];
    render(HomePage, {
      props: {
        data: {
          ...defaultData,
          categoriesTree: [food, drinks],
          categories: flatCategories,
          fullCategories: flatCategories,
          query: 'dai'
        } as PageData
      }
    });
    expect(screen.queryByRole('button', { name: /expand/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /collapse/i })).not.toBeInTheDocument();
  });
});
