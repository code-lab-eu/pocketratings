import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LoginPage from '../../src/routes/login/+page.svelte';

const mocks = vi.hoisted(() => ({
	goto: vi.fn(),
	login: vi.fn()
}));

vi.mock('$app/navigation', () => ({ goto: mocks.goto }));
vi.mock('$lib/api', () => ({ login: mocks.login }));

describe('Login page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
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
