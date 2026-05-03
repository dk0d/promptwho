<script lang="ts">
	import { Badge } from '$lib/shadcn/components/ui/badge';
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
	import { Separator } from '$lib/shadcn/components/ui/separator';
	import type { DashboardMessage, DashboardSession } from '$lib/promptwho';
	import { formatDashboardDate } from '$lib/promptwho';

	let {
		messages,
		selectedSession
	}: {
		messages: DashboardMessage[];
		selectedSession: DashboardSession | null;
	} = $props();
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between gap-2">
			<span>Messages</span>
			<Badge variant="secondary">{messages.length}</Badge>
		</CardTitle>
		<CardDescription>
			{#if selectedSession}
				Conversation timeline for {selectedSession.id}.
			{:else}
				Choose a session to inspect its conversation history.
			{/if}
		</CardDescription>
	</CardHeader>
	<CardContent class="pt-0">
		<ScrollArea class="h-[32rem] pr-3">
			{#if messages.length === 0}
				<p class="text-sm text-muted-foreground">No messages loaded for the current session.</p>
			{:else}
				<div class="space-y-4">
					{#each messages as message, index}
						<div class="rounded-lg border bg-muted/20 p-4">
							<div class="flex items-center justify-between gap-3">
								<div class="flex items-center gap-3">
									<Badge variant={message.role === 'user' ? 'default' : 'secondary'}>{message.role}</Badge>
									<span class="text-xs text-muted-foreground">{formatDashboardDate(message.created_at)}</span>
								</div>
								<DetailsDialog
									title={`Message ${message.id}`}
									description="Complete message payload from the selected session."
									data={message}
								/>
							</div>
							<p class="mt-3 whitespace-pre-wrap break-words text-sm leading-6">{message.content}</p>
							{#if index < messages.length - 1}
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
