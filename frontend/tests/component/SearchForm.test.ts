import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import SearchForm from '../../src/lib/SearchForm.svelte';

describe('SearchForm', () => {
	it('renders label and input with placeholder', () => {
		render(SearchForm, {
			props: { actionUrl: '/', query: '' }
		});
		expect(screen.getByLabelText(/search categories and products/i)).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/search categories and products/i)).toBeInTheDocument();
	});

	it('form has given action URL', () => {
		render(SearchForm, {
			props: { actionUrl: '/categories/abc-123', query: '' }
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '/categories/abc-123');
	});

	it('input value reflects query prop', () => {
		render(SearchForm, {
			props: { actionUrl: '/', query: 'milk' }
		});
		const input = screen.getByRole('searchbox');
		expect(input).toHaveValue('milk');
	});

	it('uses fallback action when actionUrl is not a relative path', () => {
		render(SearchForm, {
			props: { actionUrl: 'https://evil.com', query: '' }
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '#');
	});

	it('uses fallback action for protocol-relative URL', () => {
		render(SearchForm, {
			props: { actionUrl: '//evil.com', query: '' }
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '#');
	});

	it('uses custom placeholder when provided', () => {
		render(SearchForm, {
			props: {
				actionUrl: '/categories/1',
				query: '',
				placeholder: 'Search in category "Beverages"'
			}
		});
		expect(screen.getByPlaceholderText('Search in category "Beverages"')).toBeInTheDocument();
		expect(screen.getByLabelText('Search in category "Beverages"')).toBeInTheDocument();
	});
});
