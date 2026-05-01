import type { PageServerLoad } from './$types';
import { DEFAULT_EVENT_LIMIT } from '$lib/promptwho';
import { loadDashboardData } from '$lib/server/promptwho';

export const load: PageServerLoad = async ({ url, fetch }) => {
	const projectId = url.searchParams.get('project') ?? '';
	const sessionId = url.searchParams.get('session') ?? '';
	const action = url.searchParams.get('action') ?? '';
	const query = url.searchParams.get('q') ?? '';
	const eventLimit = url.searchParams.get('eventLimit') ?? DEFAULT_EVENT_LIMIT;

	return {
		filters: {
			projectId,
			sessionId,
			action,
			query,
			eventLimit
		},
		dashboard: await loadDashboardData(fetch, {
			projectId,
			sessionId,
			action,
			query,
			eventLimit
		})
	};
};
