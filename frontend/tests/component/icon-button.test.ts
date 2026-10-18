import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import IconButton from '../../src/lib/IconButton.svelte';

const icon = createRawSnippet(() => ({ render: () => '<span>icon</span>' }));

describe('IconButton', () => {
  it('takes its accessible name from label and is a plain button', () => {
    render(IconButton, { props: { label: 'Log out', children: icon } });
    const button = screen.getByRole('button', { name: 'Log out' });
    expect(button).toHaveAttribute('type', 'button');
  });

  it('omits aria-pressed and aria-expanded unless they are set', () => {
    render(IconButton, { props: { label: 'Plain', children: icon } });
    const button = screen.getByRole('button', { name: 'Plain' });
    expect(button).not.toHaveAttribute('aria-pressed');
    expect(button).not.toHaveAttribute('aria-expanded');
  });

  it('sets aria-pressed from pressed, including false', () => {
    render(IconButton, { props: { label: 'Toggle', pressed: false, children: icon } });
    expect(screen.getByRole('button', { name: 'Toggle' })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
  });

  it('sets aria-expanded from expanded, including false', () => {
    render(IconButton, { props: { label: 'Expand', expanded: false, children: icon } });
    expect(screen.getByRole('button', { name: 'Expand' })).toHaveAttribute(
      'aria-expanded',
      'false'
    );
  });

  it('sets the title when given', () => {
    render(IconButton, { props: { label: 'Theme', title: 'Dark mode', children: icon } });
    expect(screen.getByRole('button', { name: 'Theme' })).toHaveAttribute(
      'title',
      'Dark mode'
    );
  });

  it('renders the children inside the button', () => {
    render(IconButton, { props: { label: 'With icon', children: icon } });
    expect(screen.getByRole('button', { name: 'With icon' })).toHaveTextContent('icon');
  });

  it('calls onclick when clicked', async () => {
    const onclick = vi.fn();
    render(IconButton, { props: { label: 'Go', onclick, children: icon } });
    await userEvent.click(screen.getByRole('button', { name: 'Go' }));
    expect(onclick).toHaveBeenCalledTimes(1);
  });

  it('does not call onclick when disabled', async () => {
    const onclick = vi.fn();
    render(IconButton, { props: { label: 'Go', onclick, disabled: true, children: icon } });
    const button = screen.getByRole('button', { name: 'Go' });
    expect(button).toBeDisabled();
    await userEvent.click(button);
    expect(onclick).not.toHaveBeenCalled();
  });
});
