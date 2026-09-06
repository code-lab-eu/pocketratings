import type { PageLoad } from './$types';
import { validateResetToken } from '$lib/api';

/** Read the token from the query string and check it, so the form is only shown for a usable link. */
export const load: PageLoad = async ({ url }) => {
  const token = url.searchParams.get('token') ?? '';
  if (!token) {
    return { token: '', invalid: true };
  }
  try {
    await validateResetToken(token);
    return { token, invalid: false };
  } catch {
    return { token, invalid: true };
  }
};
