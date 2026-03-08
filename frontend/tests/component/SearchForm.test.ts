import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SearchForm from '../../src/lib/SearchForm.svelte';

const noopQueryChange = (): void => {};

describe('SearchForm', () => {
	it('renders label and input with placeholder', () => {
		render(SearchForm, {
			props: { actionUrl: '/', query: '', onQueryChange: noopQueryChange }
		});
		expect(screen.getByLabelText(/search categories and products/i)).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/search categories and products/i)).toBeInTheDocument();
	});

	it('form has given action URL', () => {
		render(SearchForm, {
			props: {
				actionUrl: '/categories/abc-123',
				query: '',
				onQueryChange: noopQueryChange
			}
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '/categories/abc-123');
	});

	it('input value reflects query prop', () => {
		render(SearchForm, {
			props: { actionUrl: '/', query: 'milk', onQueryChange: noopQueryChange }
		});
		const input = screen.getByRole('searchbox');
		expect(input).toHaveValue('milk');
	});

	it('uses fallback action when actionUrl is not a relative path', () => {
		render(SearchForm, {
			props: {
				actionUrl: 'https://evil.com',
				query: '',
				onQueryChange: noopQueryChange
			}
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '#');
	});

	it('uses fallback action for protocol-relative URL', () => {
		render(SearchForm, {
			props: {
				actionUrl: '//evil.com',
				query: '',
				onQueryChange: noopQueryChange
			}
		});
		const form = screen.getByRole('search').closest('form');
		expect(form).toHaveAttribute('action', '#');
	});

	it('uses custom placeholder when provided', () => {
		render(SearchForm, {
			props: {
				actionUrl: '/categories/1',
				query: '',
				onQueryChange: noopQueryChange,
				placeholder: 'Search in category "Beverages"'
			}
		});
		expect(screen.getByPlaceholderText('Search in category "Beverages"')).toBeInTheDocument();
		expect(screen.getByLabelText('Search in category "Beverages"')).toBeInTheDocument();
	});

	it('calls onQueryChange with trimmed value after debounce when length >= 2', async () => {
		vi.useFakeTimers();
		const onQueryChange = vi.fn();
		render(SearchForm, {
			props: { actionUrl: '/', query: '', onQueryChange }
		});
		const input = screen.getByRole('searchbox') as HTMLInputElement;
		input.value = 'milk';
		input.dispatchEvent(new Event('input', { bubbles: true }));
		expect(onQueryChange).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(200);
		expect(onQueryChange).toHaveBeenCalledTimes(1);
		expect(onQueryChange).toHaveBeenCalledWith('milk');
		vi.useRealTimers();
	});

	it('does not call onQueryChange when trimmed length is 1', async () => {
		vi.useFakeTimers();
		const onQueryChange = vi.fn();
		render(SearchForm, {
			props: { actionUrl: '/', query: '', onQueryChange }
		});
		const input = screen.getByRole('searchbox') as HTMLInputElement;
		input.value = 'm';
		input.dispatchEvent(new Event('input', { bubbles: true }));
		await vi.advanceTimersByTimeAsync(200);
		expect(onQueryChange).not.toHaveBeenCalled();
		vi.useRealTimers();
	});

	it('calls onQueryChange with empty string when user clears input', async () => {
		vi.useFakeTimers();
		const onQueryChange = vi.fn();
		render(SearchForm, {
			props: { actionUrl: '/', query: 'ab', onQueryChange }
		});
		const input = screen.getByRole('searchbox') as HTMLInputElement;
		input.value = '';
		input.dispatchEvent(new Event('input', { bubbles: true }));
		await vi.advanceTimersByTimeAsync(200);
		expect(onQueryChange).toHaveBeenCalledTimes(1);
		expect(onQueryChange).toHaveBeenCalledWith('');
		vi.useRealTimers();
	});
});
