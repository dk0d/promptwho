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
		projectCount,
		sessionCount,
		eventCount
	}: {
		baseUrl: string;
		projectCount: number;
		sessionCount: number;
		eventCount: number;
	} = $props();
</script>

<Card>
	<CardHeader class="gap-4 lg:flex-row lg:items-start lg:justify-between">
		<div class="space-y-3">
			<Badge variant="outline" class="w-fit gap-2">
				<Sparkles class="size-3.5" />
				PromptWho Dashboard
			</Badge>
			<div class="space-y-2">
				<CardTitle class="text-3xl sm:text-4xl">Query your AI-assisted work</CardTitle>
				<CardDescription class="max-w-3xl text-sm sm:text-base">
					Search prompts and outputs, inspect session timelines, and explore the raw event stream from
					<code>{baseUrl}</code>.
				</CardDescription>
			</div>
		</div>

		<div class="grid min-w-full gap-3 sm:grid-cols-3 lg:min-w-96">
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-4">
					<FolderTree class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Projects</p>
						<p class="text-2xl font-semibold">{projectCount}</p>
					</div>
				</CardContent>
			</Card>
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-4">
					<MessagesSquare class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Sessions</p>
						<p class="text-2xl font-semibold">{sessionCount}</p>
					</div>
				</CardContent>
			</Card>
			<Card class="py-0">
				<CardContent class="flex items-center gap-3 px-4 py-4">
					<Activity class="size-4 text-muted-foreground" />
					<div>
						<p class="text-xs text-muted-foreground">Events</p>
						<p class="text-2xl font-semibold">{eventCount}</p>
					</div>
				</CardContent>
			</Card>
		</div>
	</CardHeader>
</Card>
