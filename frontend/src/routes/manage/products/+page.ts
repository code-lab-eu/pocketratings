import type { PageLoad } from './$types';
import { listProducts } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async ({ url }) => {
  const categoryId = url.searchParams.get('category_id') ?? undefined;
  try {
    const products = await listProducts(categoryId ? { category_id: categoryId } : undefined);
    return { products, error: null };
  } catch (e) {
    return {
      products: [],
      error: errorMessage(e)
    };
  }
};
