import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StarRating from '../../src/lib/StarRating.svelte';

describe('StarRating', () => {
  it('renders accessible label with score for screen readers', () => {
    render(StarRating, { props: { score: 4 } });
    expect(screen.getByLabelText(/rating: 4\.0 out of 5/i)).toBeInTheDocument();
  });

  it('renders aria-label with one-decimal score (e.g. 4.25)', () => {
    render(StarRating, { props: { score: 4.25 } });
    const el = document.querySelector('[aria-label]');
    expect(el?.getAttribute('aria-label')).toMatch(/4\.3|4\.2/);
    expect(el?.getAttribute('aria-label')).toMatch(/out of 5|5 stars/i);
  });

  it('hides star graphics from screen readers (aria-hidden on visual container)', () => {
    render(StarRating, { props: { score: 3.5 } });
    const container = screen.getByLabelText(/rating: 3\.5 out of 5/i);
    expect(container.getAttribute('aria-hidden')).not.toBe('true');
    const hidden = container.querySelector('[aria-hidden="true"]');
    expect(hidden).toBeInTheDocument();
  });

  it('does not render when score is null', () => {
    const { container } = render(StarRating, { props: { score: null } });
    expect(container.querySelector('[aria-label]')).not.toBeInTheDocument();
  });

  it('does not render when score is undefined', () => {
    const { container } = render(StarRating, { props: {} });
    expect(container.querySelector('[aria-label]')).not.toBeInTheDocument();
  });

  it('renders full 5 for score 5', () => {
    render(StarRating, { props: { score: 5 } });
    expect(screen.getByLabelText(/rating: 5\.0 out of 5/i)).toBeInTheDocument();
  });
});
