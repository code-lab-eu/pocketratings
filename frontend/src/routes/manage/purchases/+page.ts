import type { PageLoad } from './$types';
import { listPurchases } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const purchases = await listPurchases();
		return { purchases, error: null };
	} catch (e) {
		return {
			purchases: [],
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
