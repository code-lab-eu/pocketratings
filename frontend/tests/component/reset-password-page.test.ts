import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ResetPasswordPage from '../../src/routes/reset-password/+page.svelte';

const mocks = vi.hoisted(() => ({
  goto: vi.fn(),
  invalidateAll: vi.fn(),
  resetPassword: vi.fn(),
  clearToken: vi.fn()
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto, invalidateAll: mocks.invalidateAll }));
vi.mock('$lib/api', () => ({ resetPassword: mocks.resetPassword }));
vi.mock('$lib/auth', () => ({ clearToken: mocks.clearToken }));

const validData = { token: 'abc123', status: 'ok' as const };

/** Fill both password fields and submit the form. */
async function submit(newPassword: string, confirmPassword: string) {
  if (newPassword) {
    await userEvent.type(screen.getByLabelText('New password'), newPassword);
  }
  if (confirmPassword) {
    await userEvent.type(screen.getByLabelText('Confirm password'), confirmPassword);
  }
  await userEvent.click(screen.getByRole('button', { name: /set password/i }));
}

describe('Reset password page', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders both password fields and a submit button for a valid link', () => {
    render(ResetPasswordPage, { props: { data: validData } });
    expect(screen.getByLabelText('New password')).toBeInTheDocument();
    expect(screen.getByLabelText('Confirm password')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /set password/i })).toBeInTheDocument();
  });

  it('shows the invalid-link message and no form when the link is invalid', () => {
    render(ResetPasswordPage, { props: { data: { token: '', status: 'invalid' as const } } });
    expect(
      screen.getByText(
        'This reset link is invalid or has expired. Ask the administrator who sent it for a new link.'
      )
    ).toBeInTheDocument();
    expect(screen.queryByLabelText('New password')).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /set password/i })).not.toBeInTheDocument();
  });

  it('offers a retry and keeps the link alive when the API cannot be reached', async () => {
    render(ResetPasswordPage, {
      props: { data: { token: 'abc123', status: 'unavailable' as const } }
    });
    expect(
      screen.getByText(
        'The server could not be reached. Your link is still valid, so check your connection and try again.'
      )
    ).toBeInTheDocument();
    expect(screen.queryByLabelText('New password')).not.toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));

    expect(mocks.invalidateAll).toHaveBeenCalled();
  });

  it('shows a validation error and does not call the API when the new password is empty', async () => {
    render(ResetPasswordPage, { props: { data: validData } });

    await submit('', '');

    expect(await screen.findByRole('alert')).toHaveTextContent('New password is required.');
    expect(mocks.resetPassword).not.toHaveBeenCalled();
  });

  it('shows a validation error when the confirmation is empty', async () => {
    render(ResetPasswordPage, { props: { data: validData } });

    await submit('newsecret', '');

    expect(await screen.findByRole('alert')).toHaveTextContent('Confirm password is required.');
    expect(mocks.resetPassword).not.toHaveBeenCalled();
  });

  it('shows a validation error when the passwords do not match', async () => {
    render(ResetPasswordPage, { props: { data: validData } });

    await submit('newsecret', 'different');

    expect(await screen.findByRole('alert')).toHaveTextContent('Passwords do not match.');
    expect(mocks.resetPassword).not.toHaveBeenCalled();
  });

  it('submits the token and password, clears any stored session, then navigates to login', async () => {
    mocks.resetPassword.mockResolvedValueOnce(undefined);
    render(ResetPasswordPage, { props: { data: validData } });

    await submit('newsecret', 'newsecret');

    expect(mocks.resetPassword).toHaveBeenCalledWith('abc123', 'newsecret');
    expect(mocks.clearToken).toHaveBeenCalled();
    expect(mocks.goto).toHaveBeenCalledWith('/login?reset=1');
  });

  it('shows the API error message and stays on the page when the reset fails', async () => {
    mocks.resetPassword.mockRejectedValueOnce(new Error('Invalid or expired reset link.'));
    render(ResetPasswordPage, { props: { data: validData } });

    await submit('newsecret', 'newsecret');

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Invalid or expired reset link.'
    );
    expect(mocks.goto).not.toHaveBeenCalled();
  });

  it('each password field can be revealed with its own toggle', async () => {
    render(ResetPasswordPage, { props: { data: validData } });
    const toggles = screen.getAllByRole('button', { name: 'Show password' });
    expect(toggles).toHaveLength(2);

    await userEvent.click(toggles[0]);

    expect(screen.getByLabelText('New password')).toHaveAttribute('type', 'text');
    expect(screen.getByLabelText('Confirm password')).toHaveAttribute('type', 'password');
  });
});
