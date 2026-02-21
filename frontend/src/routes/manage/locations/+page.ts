import type { PageLoad } from './$types';
import { listLocations } from '$lib/api';

export const load: PageLoad = async () => {
	try {
		const locations = await listLocations();
		return { locations, error: null };
	} catch (e) {
		return {
			locations: [],
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
