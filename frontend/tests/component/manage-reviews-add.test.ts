import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AddReviewPage from '../../src/routes/manage/reviews/add/+page.svelte';
import type { Product } from '../../src/lib/types';

const mocks = vi.hoisted(() => ({
  goto: vi.fn().mockResolvedValue(undefined),
  createReview: vi.fn().mockResolvedValue(undefined),
  resolve: (path: string) => path
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/paths', () => ({ resolve: mocks.resolve }));
vi.mock('$lib/api', () => ({ createReview: mocks.createReview }));

const product: Product = {
  id: 'prod-123',
  name: 'Test Product',
  brand: 'Brand',
  category: { id: 'cat-1', name: 'Cat', ancestors: [] },
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

describe('Add review page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.createReview.mockResolvedValue(undefined);
    mocks.goto.mockResolvedValue(undefined);
  });

  it('renders Add review heading and form', () => {
    render(AddReviewPage, {
      props: {
        data: { products: [product], productId: undefined, error: null }
      }
    });
    expect(screen.getByRole('heading', { name: /add review/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/product/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
  });

  it('on successful submit redirects to product page', async () => {
    render(AddReviewPage, {
      props: {
        data: { products: [product], productId: 'prod-123', error: null }
      }
    });
    await userEvent.click(screen.getByRole('button', { name: /save/i }));

    expect(mocks.createReview).toHaveBeenCalledWith(
      expect.objectContaining({ product_id: 'prod-123', rating: 4 })
    );
    expect(mocks.goto).toHaveBeenCalledWith('/products/prod-123', { invalidateAll: true });
  });
});
