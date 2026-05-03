<script lang="ts">
	import { Activity, FolderTree, MessagesSquare, Sparkles } from '@lucide/svelte';
	import { Badge } from '$lib/shadcn/components/ui/badge';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/shadcn/components/ui/card';

	let {
		baseUrl,
		error,
		projectCount,
		sessionCount,
		eventCount
	}: {
		baseUrl: string;
		error: string | null;
		projectCount: number;
		sessionCount: number;
		eventCount: number;
	} = $props();
</script>

<Card>
	<CardHeader class="gap-5 lg:flex-row lg:items-end lg:justify-between">
		<div class="space-y-3">
			<Badge variant="outline" class="w-fit gap-2">
				<Sparkles class="size-3.5" />
				PromptWho Dashboard
			</Badge>
			<div class="space-y-2">
				<CardTitle class="text-2xl sm:text-3xl">Query your AI-assisted work</CardTitle>
				<CardDescription class="max-w-3xl text-sm sm:text-base">
					Search prompts and outputs, inspect session timelines, and explore the raw event stream from
					<code>{baseUrl}</code>.
				</CardDescription>
				{#if error}
					<p class="text-sm text-destructive">The latest dashboard refresh failed.</p>
				{/if}
			</div>
		</div>

		<div class="grid min-w-full gap-3 sm:grid-cols-3 lg:min-w-[28rem] lg:max-w-xl">
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-3">
					<FolderTree class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Projects</p>
						<p class="text-xl font-semibold">{projectCount}</p>
					</div>
				</CardContent>
			</Card>
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-3">
					<MessagesSquare class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Sessions</p>
						<p class="text-xl font-semibold">{sessionCount}</p>
					</div>
				</CardContent>
			</Card>
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-3">
					<Activity class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Events</p>
						<p class="text-xl font-semibold">{eventCount}</p>
					</div>
				</CardContent>
			</Card>
		</div>
	</CardHeader>
</Card>
