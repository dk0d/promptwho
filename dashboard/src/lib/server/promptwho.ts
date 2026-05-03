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

export interface OkResult<T> {
	readonly ok: true
	readonly data: T
}
export interface ErrResult<E> {
	readonly ok: false
	readonly error: E
}
export type Result<T, E> = OkResult<T> | ErrResult<E>

export const Result = {
	ok: <T>(data: T): OkResult<T> => ({ ok: true, data }),
	err: <E>(error: E): ErrResult<E> => ({ ok: false, error }),
}


export async function Try<T, E>(fn: () => Promise<T>): Promise<Result<T, E>> {
	try {
		if (typeof fn !== "function") {
			return Result.err("Provided argument is not a function") as ErrResult<E>;
		}
		const data = await fn() as T;
		return Result.ok(data);
	} catch (error) {
		return Result.err(error instanceof Error ? error.message : String(error)) as ErrResult<E>;
	}
}


export async function loadDashboardData(
	fetchFn: typeof fetch,
	filters: DashboardFilters,
): Promise<DashboardData> {
	const baseUrl = getPromptwhoBaseUrl();
	const result = await Try<
		{
			projects: DashboardProject[];
			sessions: DashboardSession[];
			events: DashboardEvent[];
			searchHits: DashboardSearchHit[];
			messages: DashboardMessage[];
			baseUrl: string;
			error: null;
		},
		string
	>(async () => {
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
				sort: "desc",
				sort_by: "occurred_at",
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
		return {
			projects,
			sessions,
			events,
			searchHits,
			messages,
			baseUrl,
			error: null,
		};
	});
	if (result.ok)
		return {
			...result.data,
			sessions: reconcileSelectedSession(result.data.sessions, result.data.messages, filters.sessionId),
		};
	return {
		projects: [],
		sessions: [],
		events: [],
		searchHits: [],
		messages: [],
		baseUrl,
		error: result.error,
	};

}

function reconcileSelectedSession(
	sessions: DashboardSession[],
	messages: DashboardMessage[],
	selectedSessionId: string,
) {
	if (!selectedSessionId || sessions.some((session) => session.id === selectedSessionId)) {
		return sessions;
	}

	const selectedMessages = messages.filter((message) => message.session_id === selectedSessionId);
	if (selectedMessages.length === 0) {
		return sessions;
	}

	const startedAt = selectedMessages.reduce(
		(earliest, message) => (message.created_at < earliest ? message.created_at : earliest),
		selectedMessages[0].created_at,
	);
	const endedAt = selectedMessages.reduce(
		(latest, message) => (message.created_at > latest ? message.created_at : latest),
		selectedMessages[0].created_at,
	);

	return [
		{
			id: selectedSessionId,
			project_id: "Unknown project",
			provider: "Unknown provider",
			model: "Unknown model",
			started_at: startedAt,
			ended_at: endedAt,
		},
		...sessions,
	];
}
