import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CategoryPage from '../../src/routes/categories/[id]/+page.svelte';
import type { PageData } from '../../src/routes/categories/[id]/$types';
import type { Category, Product } from '../../src/lib/types';

describe('Category page', () => {
	it('shows category name and product list', () => {
		const category: Category = {
			id: 'cat-1',
			parent_id: null,
			name: 'Beverages',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		const products: Product[] = [
			{
				id: 'prod-1',
				category_id: 'cat-1',
				brand: 'Acme',
				name: 'Milk',
				created_at: 0,
				updated_at: 0,
				deleted_at: null
			}
		];
		render(CategoryPage, {
			props: {
				data: {
					category,
					childCategories: [],
					items: [{ product: products[0], rating: 4, text: 'Good' }],
					notFound: false,
					error: null
				}
			}
		});

		expect(screen.getByRole('heading', { name: /beverages/i })).toBeInTheDocument();
		const link = screen.getByRole('link', { name: /milk/i });
		expect(link).toBeInTheDocument();
		expect(link.getAttribute('href')).toContain('/products/prod-1');
		expect(screen.getByText(/rating: 4\/5/i)).toBeInTheDocument();
	});

	it('shows back link to home', () => {
		const category: Category = {
			id: 'cat-1',
			parent_id: null,
			name: 'Food',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		render(CategoryPage, {
			props: {
				data: { category, childCategories: [], items: [], notFound: false, error: null }
			}
		});

		const homeLink = screen.getByRole('link', { name: /home/i });
		expect(homeLink).toBeInTheDocument();
		expect(homeLink.getAttribute('href')).toContain('/');
	});

	it('shows Category not found and back link when notFound is true', () => {
		render(CategoryPage, {
			props: {
				data: {
					category: null,
					childCategories: [],
					items: [],
					notFound: true,
					error: null
				} as unknown as PageData
			}
		});

		expect(screen.getByText(/category not found/i)).toBeInTheDocument();
		const backLink = screen.getByRole('link', { name: /back to home/i });
		expect(backLink).toBeInTheDocument();
		expect(backLink.getAttribute('href')).toContain('/');
	});

	it('shows error when error is set', () => {
		render(CategoryPage, {
			props: {
				data: {
					category: null,
					childCategories: [],
					items: [],
					notFound: false,
					error: 'Not found'
				}
			}
		});

		expect(screen.getByText('Not found')).toBeInTheDocument();
	});

	it('shows empty state when no products', () => {
		const category: Category = {
			id: 'cat-1',
			parent_id: null,
			name: 'Empty',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		render(CategoryPage, {
			props: {
				data: { category, childCategories: [], items: [], notFound: false, error: null }
			}
		});

		expect(screen.getByRole('heading', { name: /empty/i })).toBeInTheDocument();
		expect(screen.getByText(/no products in this category/i)).toBeInTheDocument();
	});

	it('shows child categories above product list when present', () => {
		const category: Category = {
			id: 'cat-1',
			parent_id: null,
			name: 'Food',
			created_at: 0,
			updated_at: 0,
			deleted_at: null
		};
		const childCategories: Category[] = [
			{
				id: 'cat-2',
				parent_id: 'cat-1',
				name: 'Dairy',
				created_at: 0,
				updated_at: 0,
				deleted_at: null
			}
		];
		render(CategoryPage, {
			props: {
				data: { category, childCategories, items: [], notFound: false, error: null }
			}
		});

		expect(screen.getByRole('heading', { name: /food/i })).toBeInTheDocument();
		const childLink = screen.getByRole('link', { name: /dairy/i });
		expect(childLink).toBeInTheDocument();
		expect(childLink.getAttribute('href')).toContain('/categories/cat-2');
	});
});
