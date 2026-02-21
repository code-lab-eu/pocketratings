import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import LocationsPage from '../../src/routes/manage/locations/+page.svelte';
import type { Location } from '../../src/lib/types';

const location: Location = {
	id: 'loc1',
	name: 'Store A',
	deleted_at: null
};

describe('Manage locations list', () => {
	it('renders Locations heading and New location link', () => {
		render(LocationsPage, {
			props: { data: { locations: [], error: null } }
		});
		expect(screen.getByRole('heading', { name: /locations/i })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: /new location/i })).toBeInTheDocument();
	});

	it('shows empty state when no locations', () => {
		render(LocationsPage, {
			props: { data: { locations: [], error: null } }
		});
		expect(screen.getByText(/no locations yet/i)).toBeInTheDocument();
	});

	it('shows location list with edit link and delete button', () => {
		render(LocationsPage, {
			props: { data: { locations: [location], error: null } }
		});
		expect(screen.getByRole('link', { name: 'Store A' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: /delete store a/i })).toBeInTheDocument();
	});
});
