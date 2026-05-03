export interface DashboardProject {
	id: string;
	name: string | null;
	root: string;
	created_at: string;
}

export interface DashboardSession {
	id: string;
	project_id: string;
	provider: string;
	model: string;
	started_at: string;
	ended_at: string | null;
}

export interface DashboardMessage {
	id: string;
	session_id: string;
	role: string;
	content: string;
	token_count: number | null;
	created_at: string;
}

export interface DashboardEvent {
	id: string;
	project_id: string;
	session_id: string | null;
	occurred_at: string;
	action: string;
}

export interface DashboardSearchHit {
	kind: string;
	id: string;
	title: string;
	snippet: string | null;
	score: number;
}

export interface DashboardData {
	projects: DashboardProject[];
	sessions: DashboardSession[];
	messages: DashboardMessage[];
	events: DashboardEvent[];
	searchHits: DashboardSearchHit[];
	baseUrl: string;
	error: string | null;
}

export interface DashboardFilters {
	projectId: string;
	sessionId: string;
	action: string;
	query: string;
	eventLimit: string;
}

export const DEFAULT_EVENT_LIMIT = '25';

export const EVENT_LIMIT_OPTIONS = ['5', '10', '25', '50', '100'] as const;

export function parseDashboardFilters(searchParams: URLSearchParams): DashboardFilters {
	return {
		projectId: searchParams.get('project') ?? '',
		sessionId: searchParams.get('session') ?? '',
		action: searchParams.get('action') ?? '',
		query: searchParams.get('q') ?? '',
		eventLimit: searchParams.get('eventLimit') ?? DEFAULT_EVENT_LIMIT
	};
}

export function writeDashboardFilters(searchParams: URLSearchParams, filters: DashboardFilters) {
	searchParams.delete('q');
	searchParams.delete('project');
	searchParams.delete('session');
	searchParams.delete('action');
	searchParams.delete('eventLimit');

	if (filters.query) searchParams.set('q', filters.query);
	if (filters.projectId) searchParams.set('project', filters.projectId);
	if (filters.sessionId) searchParams.set('session', filters.sessionId);
	if (filters.action) searchParams.set('action', filters.action);
	if (filters.eventLimit) searchParams.set('eventLimit', filters.eventLimit);
}

export function formatDashboardDate(value: string | null) {
	if (!value) return 'In progress';

	return new Intl.DateTimeFormat(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short'
	}).format(new Date(value));
}

export function trimDashboardSnippet(value: string, length = 220) {
	return value.length > length ? `${value.slice(0, length)}...` : value;
}
