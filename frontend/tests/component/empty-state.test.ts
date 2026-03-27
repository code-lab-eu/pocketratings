import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import EmptyState from '../../src/lib/EmptyState.svelte';

describe('EmptyState', () => {
  it('renders message without icon', () => {
    render(EmptyState, { props: { message: 'Nothing here.' } });
    expect(screen.getByText('Nothing here.')).toBeInTheDocument();
    expect(document.querySelector('svg')).not.toBeInTheDocument();
  });

  it('renders Lucide icon when icon prop is set', () => {
    render(EmptyState, {
      props: { message: 'Try again.', icon: 'search' }
    });
    expect(screen.getByText('Try again.')).toBeInTheDocument();
    expect(document.querySelector('svg')).toBeInTheDocument();
  });

  it('renders action link when action is set', () => {
    render(EmptyState, {
      props: {
        message: 'Empty.',
        action: { label: 'Go', href: '/manage' }
      }
    });
    const link = screen.getByRole('link', { name: /go/i });
    expect(link).toBeInTheDocument();
    expect(link.getAttribute('href')).toContain('/manage');
  });
});
