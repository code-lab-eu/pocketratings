import { render } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import ThemeToggleIcon from '../../src/lib/ThemeToggleIcon.svelte';

describe('ThemeToggleIcon', () => {
  it('marks the graphic as decorative', () => {
    const { container } = render(ThemeToggleIcon, { props: { dark: false } });
    const root = container.querySelector('.pr-theme-toggle');
    expect(root).toBeTruthy();
    expect(root?.getAttribute('aria-hidden')).toBe('true');
  });

  it('renders two SVG layers', () => {
    const { container } = render(ThemeToggleIcon, { props: { dark: false } });
    expect(container.querySelectorAll('svg')).toHaveLength(2);
  });

  it('sets data-active to moon when not dark (light UI, moon icon)', () => {
    const { container } = render(ThemeToggleIcon, { props: { dark: false } });
    expect(container.querySelector('.pr-theme-toggle')?.getAttribute('data-active')).toBe(
      'moon'
    );
  });

  it('sets data-active to sun when dark (dark UI, sun icon)', () => {
    const { container } = render(ThemeToggleIcon, { props: { dark: true } });
    expect(container.querySelector('.pr-theme-toggle')?.getAttribute('data-active')).toBe(
      'sun'
    );
  });
});
