import type { PageLoad } from './$types';
import { ApiClientError, validateResetToken } from '$lib/api';

/** Outcome of checking the link: usable, rejected by the API, or the API could not be asked. */
export type ResetLinkStatus = 'ok' | 'invalid' | 'unavailable';

/**
 * Read the token from the query string and check it, so the form is only shown for a usable link.
 *
 * Only a 401 means the token itself was rejected; a network failure or a server error leaves the
 * link untouched and is reported as "unavailable" so the user can retry instead of being told to
 * ask for a new link.
 */
export const load: PageLoad = async ({ url }) => {
  const token = url.searchParams.get('token') ?? '';
  if (!token) {
    return { token: '', status: 'invalid' as ResetLinkStatus };
  }
  try {
    await validateResetToken(token);
    return { token, status: 'ok' as ResetLinkStatus };
  } catch (e) {
    const rejected = e instanceof ApiClientError && e.status === 401;
    return { token, status: (rejected ? 'invalid' : 'unavailable') as ResetLinkStatus };
  }
};
