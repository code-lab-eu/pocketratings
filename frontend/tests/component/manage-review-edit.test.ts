import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ReviewEditPage from '../../src/routes/manage/reviews/[id]/+page.svelte';
import type { Review } from '../../src/lib/types';

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
});
