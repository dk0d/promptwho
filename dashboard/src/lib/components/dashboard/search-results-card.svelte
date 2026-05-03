<script lang="ts">
	import { Badge } from '$lib/shadcn/components/ui/badge';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/shadcn/components/ui/card';
	import { Separator } from '$lib/shadcn/components/ui/separator';
	import type { DashboardSearchHit } from '$lib/promptwho';
	import { trimDashboardSnippet } from '$lib/promptwho';

	let {
		query,
		searchHits
	}: {
		query: string;
		searchHits: DashboardSearchHit[];
	} = $props();
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between gap-2">
			<span>Search Results</span>
			<Badge variant="secondary">{searchHits.length}</Badge>
		</CardTitle>
		<CardDescription>Relevant matches across messages, sessions, and events.</CardDescription>
	</CardHeader>
	<CardContent class="pt-0">
		{#if !query}
			<p class="text-sm text-muted-foreground">Run a search to surface message content, session metadata, and matching events.</p>
		{:else}
			<div class="rounded-lg border bg-muted/10">
				<div class="flex items-center justify-between gap-3 border-b px-4 py-3">
					<p class="text-sm text-muted-foreground">
						Showing matches for <code>{query}</code>
					</p>
					<span class="text-xs text-muted-foreground">{searchHits.length} results</span>
				</div>
				<div class="max-h-[24rem] overflow-y-auto px-4 py-4">
					{#if searchHits.length === 0}
						<p class="text-sm text-muted-foreground">No search hits found for <code>{query}</code>.</p>
					{:else}
						<div class="space-y-4">
							{#each searchHits as hit, index}
								<div class="space-y-3">
									<div class="flex flex-wrap items-center gap-2">
										<Badge variant="outline">{hit.kind}</Badge>
										<span class="text-xs text-muted-foreground">score {hit.score.toFixed(1)}</span>
									</div>
									<div>
										<p class="font-medium">{hit.title}</p>
										<p class="break-all text-xs text-muted-foreground">{hit.id}</p>
									</div>
									{#if hit.snippet}
										<p class="whitespace-pre-wrap break-words text-sm text-muted-foreground">{trimDashboardSnippet(hit.snippet)}</p>
									{/if}
									{#if index < searchHits.length - 1}
										<Separator />
									{/if}
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</CardContent>
</Card>
