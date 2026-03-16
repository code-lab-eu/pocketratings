import type { PageLoad } from './$types';
import { listCategories, listProducts } from '$lib/api';
import { flattenCategories } from '$lib/categories';
import type { Category, Product } from '$lib/types';

/** Item for ProductList: product (review_score and price come from GET /api/v1/products). */
export interface ProductListItem {
  product: Product;
}

export const load: PageLoad = async ({ url }) => {
  const q = url.searchParams.get('q') ?? '';
  try {
    const [categoriesTree, products] = await Promise.all([
      listCategories(),
      q ? listProducts({ q }) : listProducts()
    ]);
    const flat = flattenCategories(categoriesTree);
    const categories =
      q.trim() === ''
        ? flat
        : flat.filter(({ category }) =>
          category.name.toLowerCase().includes(q.toLowerCase())
        );
    const fullCategories = flat;
    const items: ProductListItem[] = products.map((product) => ({ product }));
    return { categoriesTree, categories, items, query: q, error: null, fullCategories };
  } catch (e) {
    return {
      categoriesTree: [] as Category[],
      categories: [],
      items: [],
      query: q,
      error: e instanceof Error ? e.message : String(e),
      fullCategories: []
    };
  }
};
