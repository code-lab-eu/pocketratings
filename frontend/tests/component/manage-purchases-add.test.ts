import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AddPurchasePage from '../../src/routes/manage/purchases/add/+page.svelte';

describe('Add purchase page', () => {
  it('labels the price field as the price per item', () => {
    render(AddPurchasePage, {
      props: {
        data: { products: [], locations: [], productId: undefined, error: null }
      }
    });
    expect(screen.getByLabelText('Price per item (EUR)')).toBeInTheDocument();
  });
});
