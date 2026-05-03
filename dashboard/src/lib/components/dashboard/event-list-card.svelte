<script lang="ts">
	import { SquareTerminal } from "@lucide/svelte";
	import { Badge } from "$lib/shadcn/components/ui/badge";
	import DetailsDialog from "$lib/components/dashboard/details-dialog.svelte";
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle,
	} from "$lib/shadcn/components/ui/card";
	import {
		ScrollArea,
		ScrollAreaScrollbar,
	} from "$lib/shadcn/components/ui/scroll-area";
	import { Separator } from "$lib/shadcn/components/ui/separator";
	import type { DashboardEvent } from "$lib/promptwho";
	import { formatDashboardDate } from "$lib/promptwho";

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
		<CardDescription
			>Filtered raw event stream from the Rust server endpoints.</CardDescription
		>
	</CardHeader>
	<CardContent class="pt-0">
		<ScrollArea class="h-[32rem] pr-3">
			{#if events.length === 0}
				<p class="text-sm text-muted-foreground">
					No events match the current filters.
				</p>
			{:else}
				<div class="space-y-4">
					{#each events as event, index}
						<div class="rounded-lg border bg-muted/20 p-4">
							<div class="flex flex-wrap items-start justify-between gap-3">
								<div
									class="flex flex-wrap items-center justify-between gap-2 w-full"
								>
									<Badge variant="outline">{event.action}</Badge>
									<DetailsDialog
										title={`Event ${event.id}`}
										description="Complete event payload returned by the server query."
										data={event}
									/>
								</div>
								<Badge variant="secondary">{event.project_id}</Badge>
							</div>
							<div>
								<p class="break-all font-medium">{event.id}</p>
								<p class="text-xs text-muted-foreground">
									{formatDashboardDate(event.occurred_at)}
									{#if event.session_id}
										• session {event.session_id}
									{/if}
								</p>
							</div>
							{#if index < events.length - 1}
								<Separator class="mt-4" />
							{/if}
						</div>
					{/each}
				</div>
			{/if}
			<ScrollAreaScrollbar orientation="vertical" />
		</ScrollArea>
	</CardContent>
</Card>
