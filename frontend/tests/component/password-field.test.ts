import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import PasswordField from '../../src/lib/PasswordField.svelte';

describe('PasswordField', () => {
  it('renders a labelled password input that hides the value by default', () => {
    render(PasswordField, { props: { id: 'pw', label: 'New password', value: '' } });
    expect(screen.getByLabelText('New password')).toHaveAttribute('type', 'password');
  });

  it('toggle button shows the password and flips its accessible name and pressed state', async () => {
    render(PasswordField, { props: { id: 'pw', label: 'New password', value: '' } });
    const toggle = screen.getByRole('button', { name: 'Show password' });
    expect(toggle).toHaveAttribute('aria-pressed', 'false');

    await userEvent.click(toggle);

    expect(screen.getByLabelText('New password')).toHaveAttribute('type', 'text');
    const pressed = screen.getByRole('button', { name: 'Hide password' });
    expect(pressed).toHaveAttribute('aria-pressed', 'true');
  });

  it('toggling twice hides the password again', async () => {
    render(PasswordField, { props: { id: 'pw', label: 'New password', value: '' } });

    await userEvent.click(screen.getByRole('button', { name: 'Show password' }));
    await userEvent.click(screen.getByRole('button', { name: 'Hide password' }));

    expect(screen.getByLabelText('New password')).toHaveAttribute('type', 'password');
    expect(screen.getByRole('button', { name: 'Show password' })).toBeInTheDocument();
  });

  it('passes autocomplete through to the input', () => {
    render(PasswordField, {
      props: { id: 'pw', label: 'New password', value: '', autocomplete: 'new-password' }
    });
    expect(screen.getByLabelText('New password')).toHaveAttribute(
      'autocomplete',
      'new-password'
    );
  });
});
