import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import CategoryLinkList from '../../src/lib/CategoryLinkList.svelte';
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

describe('CategoryLinkList', () => {
	it('renders list of category links with correct hrefs', () => {
		const items = [
			{ category: cat1, depth: 0 },
			{ category: cat2, depth: 1 }
		];
		render(CategoryLinkList, {
			props: { items, basePath: 'categories' }
		});
		const foodLink = screen.getByRole('link', { name: 'Food' });
		const dairyLink = screen.getByRole('link', { name: 'Dairy' });
		expect(foodLink).toBeInTheDocument();
		expect(dairyLink).toBeInTheDocument();
		expect(foodLink.getAttribute('href')).toMatch(/\/categories\/c1$/);
		expect(dairyLink.getAttribute('href')).toMatch(/\/categories\/c2$/);
	});

	it('uses basePath for link hrefs', () => {
		const items = [{ category: cat1, depth: 0 }];
		render(CategoryLinkList, {
			props: { items, basePath: 'manage/categories' }
		});
		const link = screen.getByRole('link', { name: 'Food' });
		expect(link.getAttribute('href')).toMatch(/\/manage\/categories\/c1$/);
	});

	it('renders empty list when items is empty', () => {
		render(CategoryLinkList, {
			props: { items: [], basePath: 'categories' }
		});
		const list = document.querySelector('ul.space-y-2');
		expect(list).toBeInTheDocument();
		expect(list?.children).toHaveLength(0);
	});
});
