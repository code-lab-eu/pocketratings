import { beforeEach, describe, expect, it, vi } from 'vitest';
import { load } from '../src/routes/reset-password/+page';

const mocks = vi.hoisted(() => ({
  validateResetToken: vi.fn()
}));

vi.mock('$lib/api', () => ({
  validateResetToken: (...args: unknown[]) => mocks.validateResetToken(...args)
}));

/** Minimal LoadEvent-like object; load only uses url. */
function createLoadEvent(url: URL): Parameters<typeof load>[0] {
  return {
    params: {},
    url,
    fetch: vi.fn(),
    setHeaders: vi.fn(),
    parent: async () => ({}),
    depends: vi.fn(),
    data: null,
    untrack: vi.fn(),
    tracing: {},
    route: { id: null }
  } as unknown as Parameters<typeof load>[0];
}

describe('Reset password page load', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('returns the token and invalid false when the API accepts it', async () => {
    mocks.validateResetToken.mockResolvedValue(undefined);

    const result = await load(
      createLoadEvent(new URL('https://app.example/reset-password?token=abc123'))
    );

    expect(mocks.validateResetToken).toHaveBeenCalledWith('abc123');
    expect(result).toEqual({ token: 'abc123', invalid: false });
  });

  it('returns invalid true without calling the API when the token is missing', async () => {
    const result = await load(createLoadEvent(new URL('https://app.example/reset-password')));

    expect(mocks.validateResetToken).not.toHaveBeenCalled();
    expect(result).toEqual({ token: '', invalid: true });
  });

  it('returns invalid true when validation fails', async () => {
    mocks.validateResetToken.mockRejectedValue(new Error('Invalid or expired reset link.'));

    const result = await load(
      createLoadEvent(new URL('https://app.example/reset-password?token=abc123'))
    );

    expect(result).toEqual({ token: 'abc123', invalid: true });
  });
});
