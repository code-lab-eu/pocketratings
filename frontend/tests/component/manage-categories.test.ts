import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CategoriesPage from '../../src/routes/manage/categories/+page.svelte';
import type { Category } from '../../src/lib/types';

const category: Category = {
	id: 'c1',
	parent_id: null,
	name: 'Food',
	created_at: 0,
	updated_at: 0,
	deleted_at: null
};

describe('Manage categories list', () => {
	it('renders Categories heading and New category link', () => {
		render(CategoriesPage, {
			props: { data: { categories: [], error: null } }
		});
		expect(screen.getByRole('heading', { name: /categories/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /new category/i })).toBeInTheDocument();
	});

	it('shows empty state when no categories', () => {
		render(CategoriesPage, {
			props: { data: { categories: [], error: null } }
		});
		expect(screen.getByText(/no categories yet/i)).toBeInTheDocument();
	});

	it('shows category list with edit link and delete button', () => {
		render(CategoriesPage, {
			props: { data: { categories: [category], error: null } }
		});
		expect(screen.getByRole('link', { name: 'Food' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete food/i })).toBeInTheDocument();
	});

	it('shows error when load fails', () => {
		render(CategoriesPage, {
			props: { data: { categories: [], error: 'Network error' } }
		});
		expect(screen.getByText('Network error')).toBeInTheDocument();
	});
});
