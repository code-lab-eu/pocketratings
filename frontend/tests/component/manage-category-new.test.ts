import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import CategoryNewPage from '../../src/routes/manage/categories/new/+page.svelte';
import type { Category } from '../../src/lib/types';

const mocks = vi.hoisted(() => ({
  goto: vi.fn().mockResolvedValue(undefined),
  createCategory: vi.fn(),
  resolve: vi.fn((path: string, params?: { id: string }) => {
    if (path === '/categories/[id]' && params?.id) {
      return `/categories/${params.id}`;
    }
    return path;
  })
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/paths', () => ({ resolve: mocks.resolve }));
vi.mock('$lib/api', () => ({ createCategory: mocks.createCategory }));

const createdCategory: Category = {
  id: 'new-cat-id',
  ancestors: [],
  name: 'Snacks',
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

const parentCat: Category = {
  id: 'parent-1',
  ancestors: [],
  name: 'Food',
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

describe('New category page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.createCategory.mockResolvedValue(createdCategory);
    mocks.goto.mockResolvedValue(undefined);
  });

  it('renders New category heading and form', () => {
    render(CategoryNewPage, {
      props: { data: { categories: [], error: null } }
    });
    expect(screen.getByRole('heading', { name: /new category/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/name/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /create/i })).toBeInTheDocument();
  });

  it('on successful submit redirects to public category page', async () => {
    render(CategoryNewPage, {
      props: { data: { categories: [], error: null } }
    });
    await userEvent.type(screen.getByLabelText(/name/i), 'Snacks');
    await userEvent.click(screen.getByRole('button', { name: /create/i }));

    expect(mocks.createCategory).toHaveBeenCalledWith({
      name: 'Snacks',
      parent_id: null
    });
    expect(mocks.goto).toHaveBeenCalledWith('/categories/new-cat-id', { invalidateAll: true });
  });

  it('on successful submit with parent sends parent_id and redirects', async () => {
    render(CategoryNewPage, {
      props: { data: { categories: [parentCat], error: null } }
    });
    await userEvent.type(screen.getByLabelText(/name/i), 'Candy');
    await userEvent.selectOptions(screen.getByLabelText(/parent/i), 'parent-1');
    await userEvent.click(screen.getByRole('button', { name: /create/i }));

    expect(mocks.createCategory).toHaveBeenCalledWith({
      name: 'Candy',
      parent_id: 'parent-1'
    });
    expect(mocks.goto).toHaveBeenCalledWith('/categories/new-cat-id', { invalidateAll: true });
  });
});
