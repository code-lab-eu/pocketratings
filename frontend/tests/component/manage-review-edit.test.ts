import { render, screen, fireEvent } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ReviewEditPage from '../../src/routes/manage/reviews/[id]/+page.svelte';
import type { Review } from '../../src/lib/types';

const mocks = vi.hoisted(() => ({
  goto: vi.fn().mockResolvedValue(undefined),
  updateReview: vi.fn().mockResolvedValue(undefined),
  resolve: (path: string) => path
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/paths', () => ({ resolve: mocks.resolve }));
vi.mock('$lib/api', () => ({ updateReview: mocks.updateReview }));

const review: Review = {
  id: 'r1',
  product: { id: 'p1', brand: 'B', name: 'Milk' },
  user: { id: 'u1', name: 'Alice' },
  rating: 4,
  text: 'Good',
  created_at: 0,
  updated_at: 0,
  deleted_at: null
};

describe('Manage review edit page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.updateReview.mockResolvedValue(undefined);
    mocks.goto.mockResolvedValue(undefined);
  });

  it('shows Edit review heading and form when review is loaded', () => {
    render(ReviewEditPage, {
      props: {
        data: {
          review,
          error: null
        }
      }
    });

    expect(screen.getByRole('heading', { name: /edit review/i })).toBeInTheDocument();
    expect(screen.getByLabelText(/rating/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /save/i })).toBeInTheDocument();
  });

  it('shows existing rating with one decimal (3.8) in form', () => {
    const reviewWithDecimal: Review = { ...review, rating: 3.8 };
    render(ReviewEditPage, {
      props: {
        data: {
          review: reviewWithDecimal,
          error: null
        }
      }
    });
    const slider = screen.getByRole('slider') as HTMLInputElement;
    expect(slider.value).toBe('3.8');
  });

  it('submits update with one-decimal rating 3.8', async () => {
    render(ReviewEditPage, {
      props: {
        data: {
          review,
          error: null
        }
      }
    });
    const slider = screen.getByRole('slider');
    await fireEvent.input(slider, { target: { value: '3.8' } });
    await userEvent.click(screen.getByRole('button', { name: /save/i }));

    expect(mocks.updateReview).toHaveBeenCalledTimes(1);
    expect(mocks.updateReview).toHaveBeenCalledWith(
      'r1',
      expect.objectContaining({
        rating: expect.any(Number),
        text: 'Good'
      })
    );
    const payload = mocks.updateReview.mock.calls[0][1];
    expect(payload.rating).toBeCloseTo(3.8, 10);
    expect(mocks.goto).toHaveBeenCalledWith('/products/p1', { invalidateAll: true });
  });

  it('rating slider has step 0.1', () => {
    render(ReviewEditPage, {
      props: {
        data: { review, error: null }
      }
    });
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('step', '0.1');
  });
});
