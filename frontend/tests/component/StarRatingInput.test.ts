import { render, screen, fireEvent } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import StarRatingInput from '../../src/lib/StarRatingInput.svelte';

describe('StarRatingInput', () => {
  it('renders a range slider and five star icons', () => {
    const { container } = render(StarRatingInput, { props: { value: 3, id: 'r' } });
    expect(screen.getByRole('slider')).toBeInTheDocument();
    expect(container.querySelectorAll('svg')).toHaveLength(5);
  });

  it('range input has min=1, max=5, step=0.1', () => {
    render(StarRatingInput, { props: { value: 3, id: 'r' } });
    const slider = screen.getByRole('slider');
    expect(slider).toHaveAttribute('min', '1');
    expect(slider).toHaveAttribute('max', '5');
    expect(slider).toHaveAttribute('step', '0.1');
  });

  it('shows formatted value for initial value 3', () => {
    render(StarRatingInput, { props: { value: 3, id: 'r' } });
    expect(screen.getByText('3.0')).toBeInTheDocument();
  });

  it('updates displayed value when the range changes', async () => {
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

  it('exposes stable id on the range input', () => {
    render(StarRatingInput, { props: { value: 3, id: 'test-rating' } });
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
