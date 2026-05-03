import type { PageServerLoad } from './$types';
import { parseDashboardFilters } from '$lib/promptwho';
import { loadDashboardData } from '$lib/server/promptwho';

export const load: PageServerLoad = async ({ url, fetch }) => {
	const filters = parseDashboardFilters(url.searchParams);

	return {
		filters,
		dashboard: await loadDashboardData(fetch, filters)
	};
};
