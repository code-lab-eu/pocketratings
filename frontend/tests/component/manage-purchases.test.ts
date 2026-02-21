import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import PurchasesPage from '../../src/routes/manage/purchases/+page.svelte';
import type { Purchase } from '../../src/lib/types';

const purchase: Purchase = {
	id: 'pur1',
	user_id: 'u1',
	product_id: 'p1',
	location_id: 'loc1',
	quantity: 1,
	price: '2.99',
	purchased_at: 1708012800,
	deleted_at: null
};

describe('Manage purchases list', () => {
	it('renders Purchases heading and Record purchase link', () => {
		render(PurchasesPage, {
			props: {
				data: {
					purchases: [],
					productMap: new Map(),
					locationMap: new Map(),
					error: null
				}
			}
		});
		expect(screen.getByRole('heading', { name: /purchases/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /record purchase/i })).toBeInTheDocument();
	});

	it('shows empty state when no purchases', () => {
		render(PurchasesPage, {
			props: {
				data: {
					purchases: [],
					productMap: new Map(),
					locationMap: new Map(),
					error: null
				}
			}
		});
		expect(screen.getByText(/no purchases yet/i)).toBeInTheDocument();
	});

	it('shows purchase list with delete button', () => {
		const productMap = new Map([['p1', { id: 'p1', name: 'Milk', brand: 'B', category_id: 'c1', created_at: 0, updated_at: 0, deleted_at: null }]]);
		const locationMap = new Map([['loc1', { id: 'loc1', name: 'Store A', deleted_at: null }]]);
		render(PurchasesPage, {
			props: {
				data: {
					purchases: [purchase],
					productMap,
					locationMap,
					error: null
				}
			}
		});
		expect(screen.getByText(/milk/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete purchase/i })).toBeInTheDocument();
	});
});
