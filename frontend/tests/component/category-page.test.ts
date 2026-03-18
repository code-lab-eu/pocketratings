import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { within } from '@testing-library/dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import CategoryPage from '../../src/routes/categories/[id]/+page.svelte';
import type { PageData } from '../../src/routes/categories/[id]/$types';
import type { Category, Product } from '../../src/lib/types';

const getCategoryMock = vi.hoisted(() => vi.fn());
vi.mock('$lib/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/lib/api')>();
  return { ...actual, getCategory: (...args: unknown[]) => getCategoryMock(...args) };
});

describe('Category page', () => {
  beforeEach(() => {
    getCategoryMock.mockReset();
  });

  const defaultData = {
    query: '',
    notFound: false,
    error: null
  };

  it('shows category name and product list', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Beverages',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    const product: Product = {
      id: 'prod-1',
      category: { id: 'cat-1', name: 'Beverages', ancestors: [] },
      brand: 'Acme',
      name: 'Milk',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      review_score: 4,
      price: '2.99'
    };
    render(CategoryPage, {
      props: {
        data: {
          category,
          items: [{ product }],
          ...defaultData
        }
      }
    });

    expect(screen.getByRole('heading', { name: /beverages/i })).toBeInTheDocument();
    const link = screen.getByRole('link', { name: /milk/i });
    expect(link).toBeInTheDocument();
    expect(link.getAttribute('href')).toContain('/products/prod-1');
    expect(screen.getByLabelText(/rating: 4\.0 out of 5/i)).toBeInTheDocument();
    expect(screen.getByText('2.99 €')).toBeInTheDocument();
  });

  it('shows Add product link with category_id to new product form', () => {
    const category: Category = {
      id: 'cat-123',
      ancestors: [],
      name: 'Snacks',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const addProductLink = screen.getByRole('link', { name: /add product/i });
    expect(addProductLink).toBeInTheDocument();
    expect(addProductLink.getAttribute('href')).toContain('category_id=cat-123');
    expect(addProductLink.getAttribute('href')).toContain('/manage/products/new');
  });

  it('shows breadcrumb with Home and category name', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const homeLink = screen.getByRole('link', { name: /home/i });
    expect(homeLink).toBeInTheDocument();
    expect(homeLink.getAttribute('href')).toContain('/');
    expect(screen.getByRole('heading', { name: /food/i })).toBeInTheDocument();
  });

  it('shows Category not found and back link when notFound is true', () => {
    render(CategoryPage, {
      props: {
        data: {
          category: null,
          items: [],
          query: '',
          notFound: true,
          error: null
        } as unknown as PageData
      }
    });

    expect(screen.getByText(/category not found/i)).toBeInTheDocument();
    const backLink = screen.getByRole('link', { name: /back to home/i });
    expect(backLink).toBeInTheDocument();
    expect(backLink.getAttribute('href')).toContain('/');
  });

  it('shows error when error is set', () => {
    render(CategoryPage, {
      props: {
        data: {
          category: null,
          items: [],
          query: '',
          notFound: false,
          error: 'Not found'
        }
      }
    });

    expect(screen.getByText('Not found')).toBeInTheDocument();
  });

  it('shows empty state when no products', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Empty',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    expect(screen.getByRole('heading', { name: /empty/i })).toBeInTheDocument();
    expect(screen.getByText(/no products in this category/i)).toBeInTheDocument();
  });

  it('shows child categories above product list when present', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      children: [
        {
          id: 'cat-2',
          ancestors: [{ id: 'cat-1', name: 'Food' }],
          name: 'Dairy',
          created_at: 0,
          updated_at: 0,
          deleted_at: null
        }
      ]
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    expect(screen.getByRole('heading', { name: /food/i })).toBeInTheDocument();
    const childLink = screen.getByRole('link', { name: /dairy/i });
    expect(childLink).toBeInTheDocument();
    expect(childLink.getAttribute('href')).toContain('/categories/cat-2');
  });

  it('shows expand control for child category that has children and expands to show nested category', async () => {
    const nested: Category = {
      id: 'cat-3',
      ancestors: [{ id: 'cat-2', name: 'Dairy' }, { id: 'cat-1', name: 'Food' }],
      name: 'Cheese',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      children: []
    };
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      children: [
        {
          id: 'cat-2',
          ancestors: [{ id: 'cat-1', name: 'Food' }],
          name: 'Dairy',
          created_at: 0,
          updated_at: 0,
          deleted_at: null,
          children: [nested]
        }
      ]
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const expandButton = screen.getByRole('button', { name: /expand dairy/i });
    expect(expandButton).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /cheese/i })).not.toBeInTheDocument();

    await userEvent.click(expandButton);
    const cheeseLink = screen.getByRole('link', { name: /cheese/i });
    expect(cheeseLink).toBeInTheDocument();
    expect(cheeseLink.getAttribute('href')).toContain('/categories/cat-3');
  });

  it('does not show expand button for leaf category (no children in data)', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null,
      children: [
        {
          id: 'cat-2',
          ancestors: [{ id: 'cat-1', name: 'Food' }],
          name: 'Dairy',
          created_at: 0,
          updated_at: 0,
          deleted_at: null,
          children: []
        }
      ]
    };

    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    expect(screen.queryByRole('button', { name: /expand dairy/i })).not.toBeInTheDocument();
    const dairyLink = screen.getByRole('link', { name: /dairy/i });
    expect(dairyLink.querySelector('[data-testid="chevron-spacer"]')).toBeInTheDocument();
  });

  it('shows breadcrumb with ancestor link when category has ancestors', () => {
    const category: Category = {
      id: 'cat-2',
      ancestors: [{ id: 'cat-1', name: 'Food' }],
      name: 'Dairy',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const homeLink = screen.getByRole('link', { name: /home/i });
    expect(homeLink).toBeInTheDocument();
    const foodLink = screen.getByRole('link', { name: /food/i });
    expect(foodLink).toBeInTheDocument();
    expect(foodLink.getAttribute('href')).toContain('/categories/cat-1');
    expect(screen.getByRole('heading', { name: /dairy/i })).toBeInTheDocument();
  });

  it('shows full breadcrumb in nav with correct order, links, capitalization and current page', () => {
    // Deep nesting: Home / Food / Dairy / Cheese / Goat cheese (current)
    // API returns ancestors closest-first: Cheese, Dairy, Food
    const category: Category = {
      id: 'goat-cheese-id',
      ancestors: [
        { id: 'cheese-id', name: 'Cheese' },
        { id: 'dairy-id', name: 'Dairy' },
        { id: 'food-id', name: 'Food' }
      ],
      name: 'Goat cheese',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    expect(nav).toBeInTheDocument();

    // All breadcrumb links must be inside the nav
    const navWithin = within(nav);
    const homeLink = navWithin.getByRole('link', { name: 'Home' });
    expect(homeLink).toHaveAttribute('href', expect.stringContaining('/'));

    const cheeseLink = navWithin.getByRole('link', { name: 'Cheese' });
    expect(cheeseLink).toHaveAttribute('href', expect.stringContaining('/categories/cheese-id'));

    const dairyLink = navWithin.getByRole('link', { name: 'Dairy' });
    expect(dairyLink).toHaveAttribute('href', expect.stringContaining('/categories/dairy-id'));

    const foodLink = navWithin.getByRole('link', { name: 'Food' });
    expect(foodLink).toHaveAttribute('href', expect.stringContaining('/categories/food-id'));

    // Correct order: Home, Food, Dairy, Cheese (links), then current page
    const links = navWithin.getAllByRole('link');
    expect(links).toHaveLength(4);
    expect(links[0]).toHaveAccessibleName('Home');
    expect(links[1]).toHaveAccessibleName('Food');
    expect(links[2]).toHaveAccessibleName('Dairy');
    expect(links[3]).toHaveAccessibleName('Cheese');

    // Current page is indicated with aria-current="page" and correct capitalization
    const currentItem = navWithin.getByRole('listitem', { current: 'page' });
    expect(currentItem).toBeInTheDocument();
    expect(currentItem).toHaveTextContent('Goat cheese');
  });

  it('does not mutate ancestors array across re-renders (breadcrumb order regression)', () => {
    const ancestors = [
      { id: 'cheese-id', name: 'Cheese' },
      { id: 'dairy-id', name: 'Dairy' },
      { id: 'food-id', name: 'Food' }
    ];
    const category: Category = {
      id: 'goat-cheese-id',
      ancestors,
      name: 'Goat cheese',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    const props = { data: { category, items: [], ...defaultData } };

    const { unmount } = render(CategoryPage, { props });
    unmount();
    render(CategoryPage, { props });

    const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
    const links = within(nav).getAllByRole('link');
    expect(links).toHaveLength(4);
    expect(links[0]).toHaveAccessibleName('Home');
    expect(links[1]).toHaveAccessibleName('Food');
    expect(links[2]).toHaveAccessibleName('Dairy');
    expect(links[3]).toHaveAccessibleName('Cheese');

    expect(ancestors[0].name).toBe('Cheese');
    expect(ancestors[2].name).toBe('Food');
  });

  it('shows search form with action to current category', () => {
    const category: Category = {
      id: 'cat-search-1',
      ancestors: [],
      name: 'Beverages',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData }
      }
    });

    const form = screen.getByRole('search').closest('form');
    expect(form).toBeInTheDocument();
    expect(form).toHaveAttribute('action', expect.stringContaining('/categories/cat-search-1'));
  });

  it('reflects query in search input value on category page', () => {
    const category: Category = {
      id: 'cat-1',
      ancestors: [],
      name: 'Food',
      created_at: 0,
      updated_at: 0,
      deleted_at: null
    };
    render(CategoryPage, {
      props: {
        data: { category, items: [], ...defaultData, query: 'milk' }
      }
    });

    const input = screen.getByRole('searchbox');
    expect(input).toHaveValue('milk');
  });
});
