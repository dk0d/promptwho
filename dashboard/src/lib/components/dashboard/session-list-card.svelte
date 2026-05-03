<script lang="ts">
	import { Badge } from '$lib/shadcn/components/ui/badge';
	import { Button } from '$lib/shadcn/components/ui/button';
	import DetailsDialog from '$lib/components/dashboard/details-dialog.svelte';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/shadcn/components/ui/card';
	import {
		ScrollArea,
		ScrollAreaScrollbar
	} from '$lib/shadcn/components/ui/scroll-area';
	import type { DashboardProject, DashboardSession } from '$lib/promptwho';
	import { formatDashboardDate } from '$lib/promptwho';

	let {
		sessions,
		selectedProject,
		selectedSessionId,
		onSelect
	}: {
		sessions: DashboardSession[];
		selectedProject: DashboardProject | null;
		selectedSessionId: string;
		onSelect: (sessionId: string) => void;
	} = $props();
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between gap-2">
			<span>Sessions</span>
			<Badge variant="secondary">{sessions.length}</Badge>
		</CardTitle>
		<CardDescription>
			{#if selectedProject}
				Showing sessions from {selectedProject.name ?? selectedProject.id}.
			{:else}
				Recent session slices across every project.
			{/if}
		</CardDescription>
	</CardHeader>
	<CardContent class="pt-0">
		<ScrollArea class="h-[26rem] pr-3">
			<div class="space-y-2">
				{#if sessions.length === 0}
					<p class="text-sm text-muted-foreground">No sessions match the current filters.</p>
				{:else}
					{#each sessions as session}
						<div class="flex items-start gap-2">
							<Button
								type="button"
								variant={session.id === selectedSessionId ? 'secondary' : 'outline'}
								class="h-auto flex-1 items-start justify-start px-3 py-3 text-left"
								onclick={() => onSelect(session.id === selectedSessionId ? '' : session.id)}
							>
								<span class="flex w-full flex-col gap-2">
									<span class="flex flex-wrap gap-2">
										<Badge variant="outline">{session.provider}</Badge>
										<Badge variant="outline">{session.model}</Badge>
										<Badge variant="secondary">{session.project_id}</Badge>
									</span>
									<span class="font-medium">{session.id}</span>
									<span class="text-xs text-muted-foreground">
										Started {formatDashboardDate(session.started_at)}
										{#if session.ended_at}
											 • Ended {formatDashboardDate(session.ended_at)}
										{/if}
									</span>
								</span>
							</Button>
							<DetailsDialog
								title={`Session ${session.id}`}
								description="Complete session payload from the dashboard query."
								data={session}
							/>
						</div>
					{/each}
				{/if}
			</div>
			<ScrollAreaScrollbar orientation="vertical" />
		</ScrollArea>
	</CardContent>
</Card>
