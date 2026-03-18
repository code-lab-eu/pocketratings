import type { PageLoad } from './$types';
import { listCategories } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async () => {
  try {
    const categories = await listCategories();
    return { categories, error: null };
  } catch (e) {
    return { categories: [], error: errorMessage(e) };
  }
};
