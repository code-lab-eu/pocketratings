import type { PageLoad } from './$types';
import { listReviews, me } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async () => {
  try {
    const currentUser = await me();
    const reviews = await listReviews(undefined, currentUser.user_id);
    return { reviews, error: null };
  } catch (e) {
    return {
      reviews: [],
      error: errorMessage(e)
    };
  }
};
