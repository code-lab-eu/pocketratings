import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ManagePage from '../../src/routes/manage/+page.svelte';

describe('Manage hub', () => {
	it('renders Manage heading', () => {
		render(ManagePage);
		expect(screen.getByRole('heading', { name: /manage/i })).toBeInTheDocument();
	});

	it('has links to Categories, Locations, Products, Purchases, Reviews', () => {
		render(ManagePage);
		expect(screen.getByRole('link', { name: /categories/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /locations/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /products/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /purchases/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /reviews/i })).toBeInTheDocument();
	});

	it('has link back to home', () => {
		render(ManagePage);
		expect(screen.getByRole('link', { name: /home/i })).toBeInTheDocument();
	});
});
