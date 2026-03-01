import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ReviewsPage from '../../src/routes/manage/reviews/+page.svelte';
import type { Review } from '../../src/lib/types';

const review: Review = {
	id: 'r1',
	product: { id: 'p1', brand: 'B', name: 'Milk' },
	user: { id: 'u1', name: 'Alice' },
	rating: 4,
	text: 'Good product',
	created_at: 0,
	updated_at: 0,
	deleted_at: null
};

describe('Manage reviews list', () => {
	it('renders Reviews heading and Add review link', () => {
		render(ReviewsPage, {
			props: {
				data: {
					reviews: [],
					error: null
				}
			}
		});
		expect(screen.getByRole('heading', { name: /reviews/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /add review/i })).toBeInTheDocument();
	});

	it('shows empty state when no reviews', () => {
		render(ReviewsPage, {
			props: {
				data: {
					reviews: [],
					error: null
				}
			}
		});
		expect(screen.getByText(/no reviews yet/i)).toBeInTheDocument();
	});

	it('shows review list with product link and delete button', () => {
		render(ReviewsPage, {
			props: {
				data: {
					reviews: [review],
					error: null
				}
			}
		});
		expect(screen.getByRole('link', { name: /milk/i })).toBeInTheDocument();
		expect(screen.getByText(/4\/5/)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete review/i })).toBeInTheDocument();
	});
});
