import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import EditLink from '../../src/lib/EditLink.svelte';

describe('EditLink', () => {
  it('renders a link to href named "Edit" plus the label', () => {
    render(EditLink, {
      props: { href: '/manage/products/p1', label: 'Milk — Acme' }
    });
    const link = screen.getByRole('link', { name: 'Edit Milk — Acme' });
    expect(link).toHaveAttribute('href', '/manage/products/p1');
  });
});
