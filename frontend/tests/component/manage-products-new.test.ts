import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ProductNewPage from '../../src/routes/manage/products/new/+page.svelte';
import type { Category } from '../../src/lib/types';

const categories: Category[] = [
	{
		id: 'cat-1',
		ancestors: [],
		name: 'Food',
		created_at: 0,
		updated_at: 0,
		deleted_at: null
	},
	{
		id: 'cat-2',
		ancestors: [{ id: 'cat-1', name: 'Food' }],
		name: 'Dairy',
		created_at: 0,
		updated_at: 0,
		deleted_at: null
	}
];

describe('New product page', () => {
	it('renders New product heading and form', () => {
		render(ProductNewPage, {
			props: {
				data: { categories, categoryId: null, error: null }
			}
		});

		expect(screen.getByRole('heading', { name: /new product/i })).toBeInTheDocument();
		expect(screen.getByLabelText(/name/i)).toBeInTheDocument();
		expect(screen.getByLabelText(/category/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /create/i })).toBeInTheDocument();
	});

	it('prefills category when categoryId is passed in data (e.g. from category edit)', () => {
		render(ProductNewPage, {
			props: {
				data: { categories, categoryId: 'cat-2', error: null }
			}
		});

		const categorySelect = screen.getByLabelText(/category/i);
		expect(categorySelect).toBeInTheDocument();
		// CategorySelect binds to value; when categoryId is cat-2, the select should show Dairy
		expect(categorySelect).toHaveValue('cat-2');
	});
});
