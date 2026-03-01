import type { PageLoad } from './$types';
import { listReviews } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const reviews = await listReviews();
		return { reviews, error: null };
	} catch (e) {
		return {
			reviews: [],
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
