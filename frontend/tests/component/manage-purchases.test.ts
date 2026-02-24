import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import PurchasesPage from '../../src/routes/manage/purchases/+page.svelte';
import type { Purchase } from '../../src/lib/types';

const purchase: Purchase = {
	id: 'pur1',
	user: { id: 'u1', name: 'Alice' },
	product: { id: 'p1', brand: 'B', name: 'Milk' },
	location: { id: 'loc1', name: 'Store A' },
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
					error: null
				}
			}
		});
		expect(screen.getByText(/no purchases yet/i)).toBeInTheDocument();
	});

	it('shows purchase list with delete button', () => {
		render(PurchasesPage, {
			props: {
				data: {
					purchases: [purchase],
					error: null
				}
			}
		});
		expect(screen.getByText(/milk/i)).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete purchase/i })).toBeInTheDocument();
	});
});
