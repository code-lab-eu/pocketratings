import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LoginPage from '../../src/routes/login/+page.svelte';

const mocks = vi.hoisted(() => {
  const state = { url: new URL('https://app.example/login') };
  return {
    goto: vi.fn(),
    login: vi.fn(),
    setSearch: (search: string) => {
      state.url = new URL(`https://app.example/login${search}`);
    },
    page: {
      subscribe: (run: (value: unknown) => void) => {
        run(state);
        return () => {};
      }
    }
  };
});

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$app/stores', () => ({ page: mocks.page }));
vi.mock('$lib/api', () => ({ login: mocks.login }));

describe('Login page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.setSearch('');
  });

  it('renders form with email, password and submit button', () => {
    render(LoginPage);
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
  });

  it('on submit with valid creds calls login then goto home', async () => {
    mocks.login.mockResolvedValueOnce({ token: 'jwt' });
    render(LoginPage);
    await userEvent.type(screen.getByLabelText(/email/i), 'u@example.com');
    await userEvent.type(screen.getByLabelText(/password/i), 'secret');
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }));

    expect(mocks.login).toHaveBeenCalledWith('u@example.com', 'secret');
    expect(mocks.goto).toHaveBeenCalledWith('/');
  });

  it('shows no notice when there is no query parameter', () => {
    render(LoginPage);
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('shows the session expired notice when expired=1', () => {
    mocks.setSearch('?expired=1');
    render(LoginPage);
    expect(screen.getByRole('alert')).toHaveTextContent(
      'Session expired. Please sign in again.'
    );
  });

  it('shows the password updated notice when reset=1', () => {
    mocks.setSearch('?reset=1');
    render(LoginPage);
    expect(screen.getByRole('alert')).toHaveTextContent('Password updated. Please sign in.');
  });

  it('on submit with API error shows error message', async () => {
    mocks.login.mockRejectedValueOnce(new Error('Invalid email or password'));
    render(LoginPage);
    await userEvent.type(screen.getByLabelText(/email/i), 'u@example.com');
    await userEvent.type(screen.getByLabelText(/password/i), 'wrong');
    await userEvent.click(screen.getByRole('button', { name: /sign in/i }));

    const alert = await screen.findByRole('alert');
    expect(alert).toHaveTextContent('Invalid email or password');
  });
});
