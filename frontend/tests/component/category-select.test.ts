import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import CategorySelect from '../../src/lib/CategorySelect.svelte';
import type { Category } from '../../src/lib/types';

const cat1: Category = {
	id: 'c1',
	parent_id: null,
	name: 'Food',
	created_at: 0,
	updated_at: 0,
	deleted_at: null
};

const cat2: Category = {
	id: 'c2',
	parent_id: 'c1',
	name: 'Dairy',
	created_at: 0,
	updated_at: 0,
	deleted_at: null
};

describe('CategorySelect', () => {
	it('renders label and select with options', () => {
		const options = [
			{ category: cat1, depth: 0 },
			{ category: cat2, depth: 1 }
		];
		render(CategorySelect, {
			props: {
				options,
				value: '',
				id: 'cat-select',
				label: 'Category',
				placeholder: 'Select category'
			}
		});
		expect(screen.getByLabelText(/category/i)).toBeInTheDocument();
		const select = screen.getByRole('combobox', { name: /category/i });
		expect(select).toBeInTheDocument();
		expect(select).toHaveAttribute('id', 'cat-select');
		expect(screen.getByRole('option', { name: 'Select category' })).toBeInTheDocument();
		expect(screen.getByRole('option', { name: 'Food' })).toBeInTheDocument();
		expect(screen.getByRole('option', { name: 'Dairy' })).toBeInTheDocument();
	});

	it('shows placeholder when provided', () => {
		render(CategorySelect, {
			props: {
				options: [{ category: cat1, depth: 0 }],
				value: '',
				id: 'p',
				label: 'Parent',
				placeholder: 'None'
			}
		});
		expect(screen.getByRole('option', { name: 'None' })).toBeInTheDocument();
	});

	it('omits placeholder option when placeholder is empty', () => {
		render(CategorySelect, {
			props: {
				options: [{ category: cat1, depth: 0 }],
				value: 'c1',
				id: 'p',
				label: 'Parent',
				placeholder: ''
			}
		});
		const options = screen.getByRole('combobox').querySelectorAll('option');
		expect(options).toHaveLength(1);
		expect(options[0]).toHaveValue('c1');
		expect(options[0]).toHaveTextContent('Food');
	});

	it('select reflects chosen option after user selection', async () => {
		const user = userEvent.setup();
		const options = [{ category: cat1, depth: 0 }, { category: cat2, depth: 1 }];
		render(CategorySelect, {
			props: {
				options,
				value: '',
				id: 'cat',
				label: 'Category',
				placeholder: 'Select'
			}
		});
		const select = screen.getByRole('combobox') as HTMLSelectElement;
		await user.selectOptions(select, 'c2');
		expect(select.value).toBe('c2');
	});

	it('sets required on select when required is true', () => {
		render(CategorySelect, {
			props: {
				options: [],
				value: '',
				id: 'r',
				label: 'Required',
				placeholder: '',
				required: true
			}
		});
		expect(screen.getByRole('combobox')).toBeRequired();
	});
});
