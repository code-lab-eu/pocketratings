import type { PageLoad } from './$types';
import { ApiClientError, isValidUuid, getCategory, listProducts } from '$lib/api';
import type { Product } from '$lib/types';

/** Item for ProductList: product (review_score and price from GET /api/v1/products). */
export interface ProductListItem {
  product: Product;
}

export const load: PageLoad = async ({ params, url }) => {
  const id = params.id;
  const q = url.searchParams.get('q') ?? '';
  if (!id || !isValidUuid(id)) {
    return { category: null, items: [], query: q, notFound: true, error: !id ? 'Missing category id' : null };
  }
  try {
    const [category, products] = await Promise.all([
      getCategory(id),
      listProducts({ category_id: id, ...(q.trim() && { q }) })
    ]);
    const items: ProductListItem[] = products.map((product) => ({ product }));
    return { category, items, query: q, notFound: false, error: null };
  } catch (e) {
    const notFound = e instanceof ApiClientError && e.status === 404;
    return {
      category: null,
      items: [],
      query: q,
      notFound,
      error: e instanceof Error ? e.message : String(e)
    };
  }
};
