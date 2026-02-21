import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ReviewsPage from '../../src/routes/manage/reviews/+page.svelte';
import type { Review } from '../../src/lib/types';

const review: Review = {
	id: 'r1',
	product_id: 'p1',
	user_id: 'u1',
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
					productMap: new Map(),
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
					productMap: new Map(),
					error: null
				}
			}
		});
		expect(screen.getByText(/no reviews yet/i)).toBeInTheDocument();
	});

	it('shows review list with product link and delete button', () => {
		const productMap = new Map([['p1', { id: 'p1', name: 'Milk', brand: 'B', category_id: 'c1', created_at: 0, updated_at: 0, deleted_at: null }]]);
		render(ReviewsPage, {
			props: {
				data: {
					reviews: [review],
					productMap,
					error: null
				}
			}
		});
		expect(screen.getByRole('link', { name: /milk/i })).toBeInTheDocument();
		expect(screen.getByText(/4\/5/)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete review/i })).toBeInTheDocument();
	});
});
