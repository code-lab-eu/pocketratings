import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import BackLink from '../../src/lib/BackLink.svelte';

describe('BackLink', () => {
  it('renders link with href and accessible name from label only', () => {
    render(BackLink, {
      props: { href: '/parent', label: 'Manage' }
    });
    const link = screen.getByRole('link', { name: 'Manage' });
    expect(link).toHaveAttribute('href', '/parent');
  });

  it('marks the icon svg as decorative', () => {
    const { container } = render(BackLink, {
      props: { href: '/', label: 'Home' }
    });
    const link = screen.getByRole('link', { name: 'Home' });
    const svg = link.querySelector('svg[aria-hidden="true"]');
    expect(svg).toBeTruthy();
    expect(container.querySelectorAll('svg')).toHaveLength(1);
  });
});
