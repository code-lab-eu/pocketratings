import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ProductsPage from '../../src/routes/manage/products/+page.svelte';
import type { Product } from '../../src/lib/types';

const product: Product = {
	id: 'p1',
	category: { id: 'c1', name: 'Groceries' },
	brand: 'Brand',
	name: 'Milk',
	created_at: 0,
	updated_at: 0,
	deleted_at: null
};

describe('Manage products list', () => {
	it('renders Products heading and New product link', () => {
		render(ProductsPage, {
			props: { data: { products: [], error: null } }
		});
		expect(screen.getByRole('heading', { name: /products/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /new product/i })).toBeInTheDocument();
	});

	it('shows empty state when no products', () => {
		render(ProductsPage, {
			props: { data: { products: [], error: null } }
		});
		expect(screen.getByText(/no products yet/i)).toBeInTheDocument();
	});

	it('shows product list with edit link and delete button', () => {
		render(ProductsPage, {
			props: { data: { products: [product], error: null } }
		});
		expect(screen.getByRole('link', { name: /milk/i })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete milk/i })).toBeInTheDocument();
	});
});
