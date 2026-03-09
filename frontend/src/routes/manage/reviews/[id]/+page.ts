import type { PageLoad } from './$types';
import { getReview } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
  try {
    const review = await getReview(params.id);
    return { review, error: null };
  } catch (e) {
    return {
      review: null,
      error: e instanceof Error ? e.message : String(e)
    };
  }
};
