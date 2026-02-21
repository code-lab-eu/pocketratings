import type { PageLoad } from './$types';
import { getLocation } from '$lib/api';

export const load: PageLoad = async ({ params }) => {
	const id = params.id;
	if (!id) {
		return { location: null, error: 'Missing location id' };
	}
	try {
		const location = await getLocation(id);
		return { location, error: null };
	} catch (e) {
		return {
			location: null,
			error: e instanceof Error ? e.message : String(e)
		};
	}
};
