import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CategoryEditPage from '../../src/routes/manage/categories/[id]/+page.svelte';
import type { Category } from '../../src/lib/types';

const category: Category = {
  id: 'cat-edit-1',
  ancestors: [],
  name: 'Beverages',
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

const categories: Category[] = [category];

describe('Manage category edit page', () => {
  it('shows Edit category heading and form', () => {
    render(CategoryEditPage, {
      props: {
        data: {
          category,
          categories,
          notFound: false,
          error: null
        }
      }
    });

    expect(screen.getByRole('heading', { name: /edit category/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
  });
});
