import type { PageLoad } from './$types';
import { listCategories } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async ({ url }) => {
  const categoryId = url.searchParams.get('category_id') ?? null;
  try {
    const categories = await listCategories();
    return { categories, categoryId, error: null };
  } catch (e) {
    return {
      categories: [],
      categoryId,
      error: errorMessage(e)
    };
  }
};
