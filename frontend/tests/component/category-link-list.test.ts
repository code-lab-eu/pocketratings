import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import CategoryLinkList from '../../src/lib/CategoryLinkList.svelte';
import type { Category } from '../../src/lib/types';

const cheese: Category = {
  id: 'c4',
  ancestors: [{ id: 'c2', name: 'Dairy' }, { id: 'c1', name: 'Food' }],
  name: 'Cheese',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: []
};

const dairy: Category = {
  id: 'c2',
  ancestors: [{ id: 'c1', name: 'Food' }],
  name: 'Dairy',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: [cheese]
};

const food: Category = {
  id: 'c1',
  ancestors: [],
  name: 'Food',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: [dairy]
};

const drinks: Category = {
  id: 'c3',
  ancestors: [],
  name: 'Drinks',
  created_at: 0,
  updated_at: 0,
  deleted_at: null,
  children: []
};

const testHref = (id: string) => `/categories/${id}`;
const manageHref = (id: string) => `/manage/categories/${id}`;

describe('CategoryLinkList', () => {
  describe('flat list mode', () => {
    it('renders list of category links with correct hrefs', () => {
      const items = [
        { category: food, depth: 0 },
        { category: dairy, depth: 1 }
      ];
      render(CategoryLinkList, {
        props: { items, hrefFor: testHref }
      });
      const foodLink = screen.getByRole('link', { name: 'Food' });
      const dairyLink = screen.getByRole('link', { name: 'Dairy' });
      expect(foodLink).toBeInTheDocument();
      expect(dairyLink).toBeInTheDocument();
      expect(foodLink.getAttribute('href')).toBe('/categories/c1');
      expect(dairyLink.getAttribute('href')).toBe('/categories/c2');
    });

    it('uses hrefFor for link hrefs', () => {
      const items = [{ category: food, depth: 0 }];
      render(CategoryLinkList, {
        props: { items, hrefFor: manageHref }
      });
      const link = screen.getByRole('link', { name: 'Food' });
      expect(link.getAttribute('href')).toBe('/manage/categories/c1');
    });

    it('renders empty list when items is empty', () => {
      render(CategoryLinkList, {
        props: { items: [], hrefFor: testHref }
      });
      const list = document.querySelector('ul');
      expect(list).toBeInTheDocument();
      expect(list?.children).toHaveLength(0);
    });

    it('does not show expand button when onToggle is not provided', () => {
      const items = [{ category: food, depth: 0 }];
      render(CategoryLinkList, {
        props: { items, hrefFor: testHref }
      });
      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });

    it('does not show spacer when onToggle is not provided', () => {
      const items = [{ category: drinks, depth: 0 }];
      render(CategoryLinkList, {
        props: { items, hrefFor: testHref }
      });
      const link = screen.getByRole('link', { name: 'Drinks' });
      expect(link.querySelector('[data-testid="chevron-spacer"]')).not.toBeInTheDocument();
    });
  });

  describe('tree mode', () => {
    it('shows expand button when onToggle provided and category has children', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food, drinks], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      const expandBtn = screen.getByRole('button', { name: /expand food/i });
      expect(expandBtn).toBeInTheDocument();
    });

    it('does not show expand button for leaf category but shows spacer for alignment', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [drinks], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      expect(screen.queryByRole('button')).not.toBeInTheDocument();
      const link = screen.getByRole('link', { name: 'Drinks' });
      const spacer = link.querySelector('[data-testid="chevron-spacer"]');
      expect(spacer).toBeInTheDocument();
    });

    it('uses hasChildrenOverride when provided to show or hide expand button', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: {
          tree: [drinks],
          hrefFor: testHref,
          onToggle,
          expandedIds: new Set<string>(),
          hasChildrenOverride: () => true
        }
      });
      expect(screen.getByRole('button', { name: /expand drinks/i })).toBeInTheDocument();
    });

    it('calls onToggle with category when expand button clicked', async () => {
      const user = userEvent.setup();
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      await user.click(screen.getByRole('button', { name: /expand food/i }));
      expect(onToggle).toHaveBeenCalledOnce();
      expect(onToggle).toHaveBeenCalledWith(food);
    });

    it('shows collapse button when category is in expandedIds', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set(['c1']) }
      });
      const collapseBtn = screen.getByRole('button', { name: /collapse food/i });
      expect(collapseBtn).toBeInTheDocument();
    });

    it('shows children when category is expanded', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set(['c1']) }
      });
      expect(screen.getByRole('link', { name: 'Dairy' })).toBeInTheDocument();
    });

    it('hides children when category is not expanded', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      expect(screen.queryByRole('link', { name: 'Dairy' })).not.toBeInTheDocument();
    });

    it('chevron button is inside the card link container', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      const link = screen.getByRole('link', { name: 'Food' });
      const btn = screen.getByRole('button', { name: /expand food/i });
      expect(link.contains(btn)).toBe(true);
    });

    it('all root cards share the same left edge regardless of children', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food, drinks], hrefFor: testHref, onToggle, expandedIds: new Set<string>() }
      });
      const listItems = document.querySelectorAll('ul > li');
      for (const li of listItems) {
        const style = (li as HTMLElement).style;
        expect(style.marginLeft).toBeFalsy();
      }
    });

    it('shows grandchildren when both parent and child are expanded', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set(['c1', 'c2']) }
      });
      expect(screen.getByRole('link', { name: 'Food' })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: 'Dairy' })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: 'Cheese' })).toBeInTheDocument();
    });

    it('hides grandchildren when only parent is expanded (not child)', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set(['c1']) }
      });
      expect(screen.getByRole('link', { name: 'Dairy' })).toBeInTheDocument();
      expect(screen.queryByRole('link', { name: 'Cheese' })).not.toBeInTheDocument();
    });

    it('calls onToggle with child category when child expand button clicked', async () => {
      const user = userEvent.setup();
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: testHref, onToggle, expandedIds: new Set(['c1']) }
      });
      await user.click(screen.getByRole('button', { name: /expand dairy/i }));
      expect(onToggle).toHaveBeenCalledWith(dairy);
    });

    it('passes hrefFor through to child links', () => {
      const onToggle = vi.fn();
      render(CategoryLinkList, {
        props: { tree: [food], hrefFor: manageHref, onToggle, expandedIds: new Set(['c1']) }
      });
      const link = screen.getByRole('link', { name: 'Dairy' });
      expect(link.getAttribute('href')).toBe('/manage/categories/c2');
    });
  });
});
