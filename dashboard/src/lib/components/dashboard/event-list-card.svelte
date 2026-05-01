<script lang="ts">
	import { SquareTerminal } from '@lucide/svelte';
	import { Badge } from '$lib/shadcn/components/ui/badge';
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
	import { Separator } from '$lib/shadcn/components/ui/separator';
	import type { DashboardEvent } from '$lib/promptwho';
	import { formatDashboardDate } from '$lib/promptwho';

	let { events }: { events: DashboardEvent[] } = $props();
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between gap-2">
			<span>Events</span>
			<div class="flex items-center gap-2">
				<Badge variant="secondary">{events.length}</Badge>
				<SquareTerminal class="size-4 text-muted-foreground" />
			</div>
		</CardTitle>
		<CardDescription>Filtered raw event stream from the Rust server endpoints.</CardDescription>
	</CardHeader>
	<CardContent>
		<ScrollArea class="h-[28rem] pr-3">
			{#if events.length === 0}
				<p class="text-sm text-muted-foreground">No events match the current filters.</p>
			{:else}
				<div class="space-y-4">
					{#each events as event, index}
						<div class="space-y-3">
							<div class="flex flex-wrap items-center gap-2">
								<Badge variant="outline">{event.action}</Badge>
								<Badge variant="secondary">{event.project_id}</Badge>
							</div>
							<div>
								<p class="font-medium">{event.id}</p>
								<p class="text-xs text-muted-foreground">
									{formatDashboardDate(event.occurred_at)}
									{#if event.session_id}
										 • session {event.session_id}
									{/if}
								</p>
							</div>
							{#if index < events.length - 1}
								<Separator />
							{/if}
						</div>
					{/each}
				</div>
			{/if}
			<ScrollAreaScrollbar orientation="vertical" />
		</ScrollArea>
	</CardContent>
</Card>
