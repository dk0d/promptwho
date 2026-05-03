<script lang="ts">
	import { Badge } from '$lib/shadcn/components/ui/badge';
	import { Button } from '$lib/shadcn/components/ui/button';
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
	import type { DashboardProject } from '$lib/promptwho';

	let {
		projects,
		selectedProjectId,
		onSelect
	}: {
		projects: DashboardProject[];
		selectedProjectId: string;
		onSelect: (projectId: string) => void;
	} = $props();
</script>

<Card>
	<CardHeader>
		<CardTitle class="flex items-center justify-between gap-2">
			<span>Projects</span>
			<Badge variant="secondary">{projects.length}</Badge>
		</CardTitle>
		<CardDescription>Switch between tracked repositories.</CardDescription>
	</CardHeader>
	<CardContent class="pt-0">
		<ScrollArea class="h-[22rem] pr-3">
			<div class="space-y-2">
				{#if projects.length === 0}
					<p class="text-sm text-muted-foreground">No projects are available from the promptwho server yet.</p>
				{:else}
					{#each projects as project}
						<Button
							type="button"
							variant={project.id === selectedProjectId ? 'secondary' : 'outline'}
							class="h-auto w-full items-start justify-between px-3 py-3 text-left"
							onclick={() => onSelect(project.id === selectedProjectId ? '' : project.id)}
						>
							<span class="flex min-w-0 flex-col gap-1">
								<span class="truncate font-medium">{project.name ?? project.id}</span>
								<span class="truncate text-xs text-muted-foreground">{project.root}</span>
							</span>
							<Badge variant={project.id === selectedProjectId ? 'default' : 'outline'}>
								{project.id === selectedProjectId ? 'Active' : 'View'}
							</Badge>
						</Button>
					{/each}
				{/if}
			</div>
			<ScrollAreaScrollbar orientation="vertical" />
		</ScrollArea>
	</CardContent>
</Card>
