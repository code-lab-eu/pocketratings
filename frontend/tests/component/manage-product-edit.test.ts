import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import ProductEditPage from '../../src/routes/manage/products/[id]/+page.svelte';
import type { Category, ProductDetail } from '../../src/lib/types';

const category: Category = {
  id: 'cat-1',
  ancestors: [],
  name: 'Dairy',
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

const product: ProductDetail = {
  id: 'prod-1',
  category: { id: 'cat-1', name: 'Dairy', ancestors: [] },
  brand: 'Acme',
  name: 'Milk',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  variations: [{ id: 'var-1', label: '1 l', unit: 'milliliters', quantity: 1000 }]
};

function renderPage() {
  render(ProductEditPage, {
    props: { data: { product, categories: [category], notFound: false, error: null } }
  });
  return screen.getByRole('heading', { name: /^variations$/i }).closest('section')!;
}

describe('Manage product edit page: variations', () => {
  it('opens the add variation form and closes it on Cancel', async () => {
    const section = renderPage();

    await userEvent.click(within(section).getByRole('button', { name: /^add variation$/i }));
    expect(within(section).getByLabelText(/^label$/i)).toBeInTheDocument();

    await userEvent.click(within(section).getByRole('button', { name: /^cancel$/i }));
    await waitFor(() => {
      expect(within(section).queryByLabelText(/^label$/i)).not.toBeInTheDocument();
    });
    expect(within(section).getByRole('button', { name: /^add variation$/i })).toBeInTheDocument();
  });

  it('opens the edit variation form with the current values and closes it on Cancel', async () => {
    const section = renderPage();

    await userEvent.click(within(section).getByRole('button', { name: /^edit$/i }));
    expect(within(section).getByLabelText(/^label$/i)).toHaveValue('1 l');

    await userEvent.click(within(section).getByRole('button', { name: /^cancel$/i }));
    await waitFor(() => {
      expect(within(section).queryByLabelText(/^label$/i)).not.toBeInTheDocument();
    });
    expect(within(section).getByRole('button', { name: /^edit$/i })).toBeInTheDocument();
  });
});
