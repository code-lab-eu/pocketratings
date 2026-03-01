import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import HomePage from '../../src/routes/+page.svelte';
import type { PageData } from '../../src/routes/$types';
import type { Category, Product } from '../../src/lib/types';

const defaultData: PageData = {
	categories: [],
	items: [],
	query: '',
	error: null
};

describe('Home page', () => {
	it('renders search bar with placeholder', () => {
		render(HomePage, {
			props: { data: { ...defaultData } }
		});
		expect(screen.getByLabelText(/search categories and products/i)).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/search categories and products/i)).toBeInTheDocument();
	});

	it('form action is home (submit goes to /?q=...)', () => {
		render(HomePage, {
			props: { data: { ...defaultData } }
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action');
		expect(form?.getAttribute('action')).toMatch(/\/$/);
	});

	it('shows Categories and Products sections', () => {
		render(HomePage, {
			props: { data: { ...defaultData } }
		});
		expect(screen.getByRole('heading', { name: /categories/i })).toBeInTheDocument();
		expect(screen.getByRole('heading', { name: /products/i })).toBeInTheDocument();
	});

	it('shows empty categories message when no categories', () => {
		render(HomePage, {
			props: { data: { ...defaultData } }
		});
		expect(screen.getByText(/no categories match/i)).toBeInTheDocument();
	});

	it('shows empty products message when no items', () => {
		render(HomePage, {
			props: { data: { ...defaultData } }
		});
		expect(screen.getByText(/no products match/i)).toBeInTheDocument();
	});

	it('shows category list when categories are provided', () => {
		const food: Category = {
			id: 'cat-1',
			parent_id: null,
			name: 'Food',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		const categories = [{ category: food, depth: 0 }];
		render(HomePage, {
			props: { data: { ...defaultData, categories } as PageData }
		});
		const link = screen.getByRole('link', { name: /food/i });
		expect(link).toBeInTheDocument();
		expect(link.getAttribute('href')).toContain('/categories/cat-1');
	});

	it('shows product list when items are provided', () => {
		const product: Product = {
			id: 'prod-1',
			category: { id: 'cat-1', name: 'Groceries' },
			brand: 'Acme',
			name: 'Milk',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		render(HomePage, {
			props: {
				data: {
					...defaultData,
					items: [{ product, rating: 4, text: 'Good' }]
				} as PageData
			}
		});
		const link = screen.getByRole('link', { name: /milk/i });
		expect(link).toBeInTheDocument();
		expect(link.getAttribute('href')).toContain('/products/prod-1');
		expect(screen.getByText(/rating: 4\/5/i)).toBeInTheDocument();
	});

	it('shows error message when error is set (no categories or products)', () => {
		render(HomePage, {
			props: { data: { ...defaultData, error: 'Network error' } as PageData }
		});
		expect(screen.getByText('Network error')).toBeInTheDocument();
		expect(screen.queryByRole('heading', { name: /categories/i })).not.toBeInTheDocument();
	});

	it('reflects query in search input value', () => {
		render(HomePage, {
			props: { data: { ...defaultData, query: 'milk' } as PageData }
		});
		const input = screen.getByRole('searchbox');
		expect(input).toHaveValue('milk');
	});
});
