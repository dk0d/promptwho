<script lang="ts">
	import type { PageData } from "./$types";
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import DashboardFiltersPanel from "$lib/components/dashboard/dashboard-filters.svelte";
	import DashboardHeader from "$lib/components/dashboard/dashboard-header.svelte";
	import EventListCard from "$lib/components/dashboard/event-list-card.svelte";
	import MessageListCard from "$lib/components/dashboard/message-list-card.svelte";
	import ProjectListCard from "$lib/components/dashboard/project-list-card.svelte";
	import SearchResultsCard from "$lib/components/dashboard/search-results-card.svelte";
	import SessionListCard from "$lib/components/dashboard/session-list-card.svelte";
	import { DEFAULT_EVENT_LIMIT, type DashboardFilters } from "$lib/promptwho";

	let { data }: { data: PageData } = $props();

	const projects = $derived(data.dashboard.projects);
	const sessions = $derived(data.dashboard.sessions);
	const messages = $derived(data.dashboard.messages);
	const events = $derived(data.dashboard.events);
	const searchHits = $derived(data.dashboard.searchHits);

	const selectedProject = $derived(
		projects.find((project) => project.id === data.filters.projectId) ?? null,
	);

	const selectedSession = $derived(
		sessions.find((session) => session.id === data.filters.sessionId) ?? null,
	);

	let form = $state<DashboardFilters>({
		query: "",
		projectId: "",
		sessionId: "",
		action: "",
		eventLimit: DEFAULT_EVENT_LIMIT,
	});

	$effect(() => {
		form = {
			query: data.filters.query,
			projectId: data.filters.projectId,
			sessionId: data.filters.sessionId,
			action: data.filters.action,
			eventLimit: data.filters.eventLimit,
		};
	});

	async function applyFilters(next?: Partial<DashboardFilters>) {
		const merged = { ...form, ...next };
		const url = new URL(page.url);

		url.searchParams.delete("q");
		url.searchParams.delete("project");
		url.searchParams.delete("session");
		url.searchParams.delete("action");
		url.searchParams.delete("eventLimit");

		if (merged.query) url.searchParams.set("q", merged.query);
		if (merged.projectId) url.searchParams.set("project", merged.projectId);
		if (merged.sessionId) url.searchParams.set("session", merged.sessionId);
		if (merged.action) url.searchParams.set("action", merged.action);
		if (merged.eventLimit)
			url.searchParams.set("eventLimit", merged.eventLimit);

		form = merged;
		await goto(`${url.pathname}${url.search}`, {
			invalidateAll: true,
			keepFocus: true,
			replaceState: true,
			noScroll: true,
		});
	}

	function onProjectChange(projectId: string) {
		form.projectId = projectId;
		form.sessionId = "";
		void applyFilters({ projectId, sessionId: "" });
	}

	function onSessionChange(sessionId: string) {
		form.sessionId = sessionId;
		void applyFilters({ sessionId });
	}

	function onEventLimitChange(eventLimit: string) {
		form.eventLimit = eventLimit;
		void applyFilters({ eventLimit });
	}

	function selectProject(projectId: string) {
		void applyFilters({ projectId, sessionId: "" });
	}

	function selectSession(sessionId: string) {
		void applyFilters({ sessionId });
	}

	function runSearch(event: SubmitEvent) {
		event.preventDefault();
		void applyFilters();
	}
</script>

<svelte:head>
	<title>PromptWho Dashboard</title>
	<meta
		name="description"
		content="PromptWho dashboard for querying projects, sessions, messages, and events from your local promptwho server."
	/>
</svelte:head>

<div class="min-h-screen bg-background text-foreground">
	<div
		class="mx-auto flex w-full max-w-7xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8"
	>
		<DashboardHeader
			baseUrl={data.dashboard.baseUrl}
			projectCount={projects.length}
			sessionCount={sessions.length}
			eventCount={events.length}
		/>

		<DashboardFiltersPanel
			{form}
			{projects}
			{sessions}
			onSubmit={runSearch}
			{onProjectChange}
			{onSessionChange}
			{onEventLimitChange}
		/>
		{#if data.dashboard.baseUrl === ""}
			<div class="rounded-lg bg-accent p-4">
				<div class="flex">
					<div class="shrink-0">
						No data available. Please set up your local PromptWho server and
						connect it to the dashboard to see your projects, sessions,
						messages, and events here.
					</div>
				</div>
			</div>
		{:else}
			<div class="grid gap-6 xl:grid-cols-[320px_minmax(0,1fr)]">
				<div class="space-y-6">
					<ProjectListCard
						{projects}
						selectedProjectId={data.filters.projectId}
						onSelect={selectProject}
					/>
					<SessionListCard
						{sessions}
						{selectedProject}
						selectedSessionId={data.filters.sessionId}
						onSelect={selectSession}
					/>
				</div>

				<div class="space-y-6">
					<SearchResultsCard query={data.filters.query} {searchHits} />

					<div class="grid gap-6 2xl:grid-cols-2">
						<MessageListCard {messages} {selectedSession} />
						<EventListCard {events} />
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>
