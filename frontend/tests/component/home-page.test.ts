import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import HomePage from '../../src/routes/+page.svelte';
import type { Category } from '../../src/lib/types';

describe('Home page', () => {
	it('renders search bar with placeholder', () => {
		render(HomePage, {
			props: { data: { categories: [], error: null } }
		});
		expect(screen.getByLabelText(/search products/i)).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/search products/i)).toBeInTheDocument();
	});

	it('shows empty state when no categories', () => {
		render(HomePage, {
			props: { data: { categories: [], error: null } }
		});
		expect(screen.getByText(/no categories yet/i)).toBeInTheDocument();
	});

	it('shows category list when categories are provided', () => {
		const categories: Category[] = [
			{
				id: 'cat-1',
				parent_id: null,
				name: 'Food',
				created_at: 0,
				updated_at: 0,
				deleted_at: null
			}
		];
		render(HomePage, {
			props: { data: { categories, error: null } }
		});
		const link = screen.getByRole('link', { name: /food/i });
		expect(link).toBeInTheDocument();
		expect(link.getAttribute('href')).toContain('/categories/cat-1');
	});

	it('shows error message when error is set', () => {
		render(HomePage, {
			props: { data: { categories: [], error: 'Network error' } }
		});
		expect(screen.getByText('Network error')).toBeInTheDocument();
	});
});
