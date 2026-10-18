import { createRawSnippet } from 'svelte';
import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Layout from '../../src/routes/+layout.svelte';
import { getToken, setToken } from '../../src/lib/auth';
import { dark } from '../../src/lib/theme';

const mocks = vi.hoisted(() => {
  const state = { url: new URL('https://app.example/') };
  return {
    goto: vi.fn().mockResolvedValue(undefined),
    resolve: vi.fn((path: string) => path),
    page: {
      subscribe: (run: (value: unknown) => void) => {
        run(state);
        return () => {};
      }
    }
  };
});

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/paths', () => ({ base: '', resolve: mocks.resolve }));
vi.mock('$app/stores', () => ({ page: mocks.page }));

const content = createRawSnippet(() => ({ render: () => '<p>page content</p>' }));

describe('Layout header', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    dark.set(false);
    setToken('jwt');
  });

  it('shows the Menu link, theme toggle and Log out button when signed in', () => {
    render(Layout, { props: { children: content } });
    expect(screen.getByRole('link', { name: 'Menu' })).toHaveAttribute('href', '/manage');
    expect(screen.getByRole('button', { name: 'Switch to dark mode' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Log out' })).toBeInTheDocument();
    expect(screen.getByText('page content')).toBeInTheDocument();
  });

  it('theme toggle name flips between dark and light mode on click', async () => {
    render(Layout, { props: { children: content } });

    await userEvent.click(screen.getByRole('button', { name: 'Switch to dark mode' }));
    expect(screen.getByRole('button', { name: 'Switch to light mode' })).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Switch to light mode' }));
    expect(screen.getByRole('button', { name: 'Switch to dark mode' })).toBeInTheDocument();
  });

  it('Log out clears the token and navigates to /login', async () => {
    render(Layout, { props: { children: content } });

    await userEvent.click(screen.getByRole('button', { name: 'Log out' }));

    expect(getToken()).toBeNull();
    expect(mocks.goto).toHaveBeenCalledWith('/login');
  });
});
