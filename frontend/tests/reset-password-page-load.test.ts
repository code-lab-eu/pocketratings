import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiClientError } from '$lib/api';
import { load } from '../src/routes/reset-password/+page';

const mocks = vi.hoisted(() => ({
  validateResetToken: vi.fn()
}));

// Keep the real ApiClientError so the loader can tell a rejected token from an unreachable API.
vi.mock('$lib/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('$lib/api')>()),
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

  it('returns the token and status ok when the API accepts it', async () => {
    mocks.validateResetToken.mockResolvedValue(undefined);

    const result = await load(
      createLoadEvent(new URL('https://app.example/reset-password?token=abc123'))
    );

    expect(mocks.validateResetToken).toHaveBeenCalledWith('abc123');
    expect(result).toEqual({ token: 'abc123', status: 'ok' });
  });

  it('returns status invalid without calling the API when the token is missing', async () => {
    const result = await load(createLoadEvent(new URL('https://app.example/reset-password')));

    expect(mocks.validateResetToken).not.toHaveBeenCalled();
    expect(result).toEqual({ token: '', status: 'invalid' });
  });

  it('returns status invalid when the API rejects the token', async () => {
    mocks.validateResetToken.mockRejectedValue(
      new ApiClientError('Invalid or expired reset link.', 401, 'unauthorized')
    );

    const result = await load(
      createLoadEvent(new URL('https://app.example/reset-password?token=abc123'))
    );

    expect(result).toEqual({ token: 'abc123', status: 'invalid' });
  });

  it.each([
    ['the network is unreachable', new TypeError('Failed to fetch')],
    ['the API errors', new ApiClientError('HTTP 500', 500, 'internal')]
  ])('returns status unavailable when %s', async (_name, error) => {
    mocks.validateResetToken.mockRejectedValue(error);

    const result = await load(
      createLoadEvent(new URL('https://app.example/reset-password?token=abc123'))
    );

    expect(result).toEqual({ token: 'abc123', status: 'unavailable' });
  });
});
