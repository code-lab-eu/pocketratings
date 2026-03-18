import type { PageLoad } from './$types';
import { listPurchases, me } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async () => {
  try {
    const currentUser = await me();
    const purchases = await listPurchases({ user_id: currentUser.user_id });
    return { purchases, error: null };
  } catch (e) {
    return {
      purchases: [],
      error: errorMessage(e)
    };
  }
};
