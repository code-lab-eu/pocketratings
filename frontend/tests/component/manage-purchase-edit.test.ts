import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import PurchaseEditPage from '../../src/routes/manage/purchases/[id]/+page.svelte';
import type { Purchase } from '../../src/lib/types';
import { pinTimezone } from '../timezone';

const purchase: Purchase = {
  id: 'pur1',
  user: { id: 'u1', name: 'Alice' },
  product: { id: 'p1', brand: 'B', name: 'Milk' },
  variation: { id: 'var1', label: '', unit: 'none' },
  location: { id: 'loc1', name: 'Store A' },
  quantity: 1,
  price: '2.99',
  purchased_at: 1708012800,
  deleted_at: null
};

describe('Manage purchase edit page', () => {
  pinTimezone('Europe/Helsinki');

  it('shows Edit purchase heading and form when purchase is loaded', () => {
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

    expect(screen.getByRole('heading', { name: /edit purchase/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/product/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
    expect(screen.getByLabelText('Price per item (EUR)')).toHaveValue('2.99');
    expect(screen.getByLabelText('Variation')).toBeRequired();
  });

  it('shows the purchase date in local time', () => {
    render(PurchaseEditPage, {
      props: { data: { purchase, products: [], locations: [], error: null } }
    });

    // purchased_at is 2024-02-15 16:00 UTC, which is 18:00 in Helsinki (UTC+2).
    expect(screen.getByLabelText('Date')).toHaveValue('2024-02-15T18:00');
  });
});
