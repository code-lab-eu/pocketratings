import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import PriceDisplay from '../../src/lib/PriceDisplay.svelte';

describe('PriceDisplay', () => {
  it('renders amount with euro symbol (e.g. 2.99 €)', () => {
    render(PriceDisplay, { props: { amount: '2.99' } });
    expect(screen.getByText('2.99 €')).toBeInTheDocument();
  });

  it('has accessible label for screen readers', () => {
    render(PriceDisplay, { props: { amount: '2.99' } });
    expect(screen.getByLabelText(/price: 2\.99 euros/i)).toBeInTheDocument();
  });

  it('renders nothing when amount is null', () => {
    const { container } = render(PriceDisplay, { props: { amount: null } });
    expect(container.firstElementChild).toBeNull();
  });

  it('renders nothing when amount is empty string', () => {
    const { container } = render(PriceDisplay, { props: { amount: '' } });
    expect(container.firstElementChild).toBeNull();
  });

  it('renders nothing when amount is undefined', () => {
    const { container } = render(PriceDisplay, { props: {} });
    expect(container.firstElementChild).toBeNull();
  });
});
