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
	import {
		parseDashboardFilters,
		writeDashboardFilters,
		type DashboardFilters,
	} from "$lib/promptwho";

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

	let form = $state<DashboardFilters>(
		parseDashboardFilters(page.url.searchParams),
	);

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

		writeDashboardFilters(url.searchParams, merged);

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

<div
	class="min-h-screen bg-[linear-gradient(180deg,color-mix(in_oklab,var(--background)_90%,white)_0%,var(--background)_16rem)] text-foreground dark:bg-[linear-gradient(180deg,color-mix(in_oklab,var(--background)_88%,black)_0%,var(--background)_18rem)]"
>
	<div
		class="mx-auto flex w-full max-w-7xl flex-col gap-6 px-4 py-6 sm:px-6 sm:py-8 lg:px-8 lg:py-10"
	>
		<DashboardHeader
			baseUrl={data.dashboard.baseUrl}
			error={data.dashboard.error}
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
		{#if data.dashboard.error}
			<div
				class="rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm"
			>
				<p class="font-medium text-destructive">
					Dashboard data could not be loaded.
				</p>
				<p class="mt-1 text-muted-foreground">
					{data.dashboard.error}
				</p>
				<p class="mt-3 text-muted-foreground">
					Checked endpoint <code>{data.dashboard.baseUrl}</code>.
				</p>
			</div>
		{:else}
			<div class="grid gap-6 xl:grid-cols-[320px_minmax(0,1fr)]">
				<div class="space-y-6 xl:sticky xl:top-6 xl:self-start">
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

				<div class="grid gap-6">
					<div
						class="grid gap-6 2xl:grid-cols-[minmax(0,1.1fr)_minmax(0,0.9fr)]"
					>
						<MessageListCard {messages} {selectedSession} />
						<EventListCard {events} />
					</div>

					<SearchResultsCard query={data.filters.query} {searchHits} />
				</div>
			</div>
		{/if}
	</div>
</div>
