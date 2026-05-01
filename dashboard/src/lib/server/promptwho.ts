import { env } from "$env/dynamic/private";
import type {
	DashboardData,
	DashboardEvent,
	DashboardFilters,
	DashboardMessage,
	DashboardProject,
	DashboardSearchHit,
	DashboardSession,
} from "$lib/promptwho";

const DEFAULT_BASE_URL = "http://127.0.0.1:8765";

export function getPromptwhoBaseUrl() {
	return env.PROMPTWHO_BASE_URL?.trim() || DEFAULT_BASE_URL;
}

async function fetchJson<T>(
	fetchFn: typeof fetch,
	path: string,
	params?: Record<string, string>,
) {
	const url = new URL(path, getPromptwhoBaseUrl());

	for (const [key, value] of Object.entries(params ?? {})) {
		if (value) {
			url.searchParams.set(key, value);
		}
	}

	const response = await fetchFn(url, {
		headers: {
			accept: "application/json",
		},
	});

	if (!response.ok) {
		throw new Error(
			`PromptWho request failed (${response.status} ${response.statusText}) for ${path}`,
		);
	}

	return (await response.json()) as T;
}

export async function loadDashboardData(
	fetchFn: typeof fetch,
	filters: DashboardFilters,
): Promise<DashboardData> {
	const baseUrl = getPromptwhoBaseUrl();
	console.log("Fetching PromptWho dashboard data with filters:", filters);
	const [projects, sessions, events, searchHits, messages] = await Promise.all([

		fetchJson<DashboardProject[]>(fetchFn, "/v1/projects"),
		fetchJson<DashboardSession[]>(fetchFn, "/v1/sessions", {
			project_id: filters.projectId,
			limit: "60",
		}),
		fetchJson<DashboardEvent[]>(fetchFn, "/v1/events/query", {
			project_id: filters.projectId,
			session_id: filters.sessionId,
			action: filters.action,
			limit: filters.eventLimit,
		}),
		filters.query
			? fetchJson<DashboardSearchHit[]>(fetchFn, "/v1/search", {
				q: filters.query,
				project_id: filters.projectId,
				limit: "30",
			})
			: Promise.resolve([]),
		filters.sessionId
			? fetchJson<DashboardMessage[]>(
				fetchFn,
				`/v1/sessions/${encodeURIComponent(filters.sessionId)}/messages`,
			)
			: Promise.resolve([]),
	]);

	const data = {
		projects,
		sessions,
		events,
		searchHits,
		messages,
		baseUrl,
	};
	console.log(data.sessions)
	return data
}
