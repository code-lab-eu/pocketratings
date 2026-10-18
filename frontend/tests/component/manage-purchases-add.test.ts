import { render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AddPurchasePage from '../../src/routes/manage/purchases/add/+page.svelte';
import { pinTimezone } from '../timezone';

describe('Add purchase page', () => {
  pinTimezone('Europe/Helsinki');

  afterEach(() => {
    vi.useRealTimers();
  });

  it('labels the price field as the price per item', () => {
    render(AddPurchasePage, {
      props: {
        data: { products: [], locations: [], productId: undefined, error: null }
      }
    });
    expect(screen.getByLabelText('Price per item (EUR)')).toBeInTheDocument();
  });

  it('defaults the date to the current local time', () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date('2024-06-15T09:30:00Z'));
    render(AddPurchasePage, {
      props: {
        data: { products: [], locations: [], productId: undefined, error: null }
      }
    });

    // 09:30 UTC is 12:30 in Helsinki (UTC+3 in summer).
    expect(screen.getByLabelText('Date')).toHaveValue('2024-06-15T12:30');
  });
});
