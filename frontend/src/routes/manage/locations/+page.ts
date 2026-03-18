import type { PageLoad } from './$types';
import { listLocations } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async () => {
  try {
    const locations = await listLocations();
    return { locations, error: null };
  } catch (e) {
    return {
      locations: [],
      error: errorMessage(e)
    };
  }
};
