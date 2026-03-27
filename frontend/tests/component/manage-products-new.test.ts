import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ProductNewPage from '../../src/routes/manage/products/new/+page.svelte';
import type { Category, Product } from '../../src/lib/types';

const mocks = vi.hoisted(() => ({
  goto: vi.fn().mockResolvedValue(undefined),
  createProduct: vi.fn(),
  resolve: vi.fn((path: string, params?: { id: string }) => {
    if (path === '/products/[id]' && params?.id) {
      return `/products/${params.id}`;
    }
    return path;
  })
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/paths', () => ({ resolve: mocks.resolve }));
vi.mock('$lib/api', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/lib/api')>();
  return { ...actual, createProduct: (...args: unknown[]) => mocks.createProduct(...args) };
});

const categories: Category[] = [
  {
    id: 'cat-1',
    ancestors: [],
    name: 'Food',
    created_at: 0,
    updated_at: 0,
    deleted_at: null
  },
  {
    id: 'cat-2',
    ancestors: [{ id: 'cat-1', name: 'Food' }],
    name: 'Dairy',
    created_at: 0,
    updated_at: 0,
    deleted_at: null
  }
];

const createdProduct: Product = {
  id: 'new-prod-id',
  name: 'Milk',
  brand: 'Acme',
  category: { id: 'cat-1', name: 'Food', ancestors: [] },
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

describe('New product page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.createProduct.mockResolvedValue(createdProduct);
    mocks.goto.mockResolvedValue(undefined);
  });

  it('renders New product heading and form', () => {
    render(ProductNewPage, {
      props: {
        data: { categories, categoryId: null, error: null }
      }
    });

    expect(screen.getByRole('heading', { name: /new product/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/^name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/category/i)).toBeInTheDocument();
    expect(screen.getByText(/first variation/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/^unit$/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /create/i })).toBeInTheDocument();
  });

  it('prefills category when categoryId is passed in data (e.g. from category edit)', () => {
    render(ProductNewPage, {
      props: {
        data: { categories, categoryId: 'cat-2', error: null }
      }
    });

    const categorySelect = screen.getByLabelText(/category/i);
    expect(categorySelect).toBeInTheDocument();
    expect(categorySelect).toHaveValue('cat-2');
  });

  it('on successful submit redirects to public product page', async () => {
    render(ProductNewPage, {
      props: { data: { categories, categoryId: null, error: null } }
    });
    await userEvent.type(screen.getByLabelText(/^name/i), 'Milk');
    await userEvent.selectOptions(screen.getByLabelText(/^category/i), 'cat-1');
    await userEvent.click(screen.getByRole('button', { name: /create/i }));

    expect(mocks.createProduct).toHaveBeenCalledWith({
      name: 'Milk',
      brand: '',
      category_id: 'cat-1'
    });
    expect(mocks.goto).toHaveBeenCalledWith('/products/new-prod-id', { invalidateAll: true });
  });
});
