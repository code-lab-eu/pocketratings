import type { PageLoad } from './$types';
import { getPurchase, listLocations, listProducts } from '$lib/api';
import { errorMessage } from '$lib/utils/formatters';

export const load: PageLoad = async ({ params }) => {
  try {
    const [purchase, products, locations] = await Promise.all([
      getPurchase(params.id),
      listProducts(),
      listLocations()
    ]);
    return { purchase, products, locations, error: null };
  } catch (e) {
    return {
      purchase: null,
      products: [],
      locations: [],
      error: errorMessage(e)
    };
  }
};
