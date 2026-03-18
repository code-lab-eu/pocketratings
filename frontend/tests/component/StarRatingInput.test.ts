import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StarRatingInput from '../../src/lib/StarRatingInput.svelte';

describe('StarRatingInput', () => {
  it('renders label text and 5 star SVGs', () => {
    const { container } = render(StarRatingInput, { props: { value: 3, id: 'r' } });
    expect(screen.getByText('Rating (1-5)')).toBeInTheDocument();
    const stars = container.querySelectorAll('svg');
    expect(stars.length).toBe(5);
  });

  it('range input has min=1, max=5, step=0.1', () => {
    render(StarRatingInput, { props: { value: 3, id: 'r' } });
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('min', '1');
    expect(slider).toHaveAttribute('max', '5');
    expect(slider).toHaveAttribute('step', '0.1');
  });

  it('floating label shows formatted value for initial value 3', () => {
    render(StarRatingInput, { props: { value: 3, id: 'r' } });
    expect(screen.getByText('3.0')).toBeInTheDocument();
  });

  it('floating label updates when range value changes', async () => {
    render(StarRatingInput, { props: { value: 3, id: 'r' } });
    const slider = screen.getByRole('slider');
    await fireEvent.input(slider, { target: { value: '4.2' } });
    expect(screen.getByText('4.2')).toBeInTheDocument();
  });

  it('renders with custom value 2.5', () => {
    render(StarRatingInput, { props: { value: 2.5, id: 'r' } });
    const slider = screen.getByRole('slider') as HTMLInputElement;
    expect(slider.value).toBe('2.5');
    expect(screen.getByText('2.5')).toBeInTheDocument();
  });

  it('accepts custom label text', () => {
    render(StarRatingInput, { props: { value: 3, id: 'r', label: 'My score' } });
    expect(screen.getByText('My score')).toBeInTheDocument();
  });

  it('associates label with the range input via id', () => {
    render(StarRatingInput, { props: { value: 3, id: 'test-rating' } });
    const label = screen.getByText('Rating (1-5)');
    expect(label).toHaveAttribute('for', 'test-rating');
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('id', 'test-rating');
  });

  it('hides star visuals from screen readers', () => {
    const { container } = render(StarRatingInput, { props: { value: 3, id: 'r' } });
    const starContainer = container.querySelector('[aria-hidden="true"]');
    expect(starContainer).toBeInTheDocument();
    const stars = starContainer?.querySelectorAll('svg');
    expect(stars?.length).toBe(5);
  });
});
