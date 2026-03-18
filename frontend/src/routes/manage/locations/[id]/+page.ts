import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getLocation } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async ({ params }) => {
  const id = params.id;
  if (!id || !isValidUuid(id)) {
    return { location: null, notFound: true, error: !id ? 'Missing location id' : null };
  }
  try {
    const location = await getLocation(id);
    return { location, notFound: false, error: null };
  } catch (e) {
    const notFound = e instanceof ApiClientError && e.status === 404;
    return {
      location: null,
      notFound,
      error: errorMessage(e)
    };
  }
};
