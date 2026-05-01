<script lang="ts">
	import { Search } from '@lucide/svelte';
	import { Button } from '$lib/shadcn/components/ui/button';
	import { Card, CardContent } from '$lib/shadcn/components/ui/card';
	import { Input } from '$lib/shadcn/components/ui/input';
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger
	} from '$lib/shadcn/components/ui/select';
	import {
		EVENT_LIMIT_OPTIONS,
		type DashboardFilters,
		type DashboardProject,
		type DashboardSession
	} from '$lib/promptwho';

	let {
		form,
		projects,
		sessions,
		onSubmit,
		onProjectChange,
		onSessionChange,
		onEventLimitChange
	}: {
		form: DashboardFilters;
		projects: DashboardProject[];
		sessions: DashboardSession[];
		onSubmit: (event: SubmitEvent) => void;
		onProjectChange: (projectId: string) => void;
		onSessionChange: (sessionId: string) => void;
		onEventLimitChange: (eventLimit: string) => void;
	} = $props();

	function truncateLabel(value: string, maxLength = 40) {
		return value.length > maxLength ? `${value.slice(0, maxLength - 1)}...` : value;
	}

	const selectedProjectLabel = $derived(
		form.projectId
			? truncateLabel(
					projects.find((project) => project.id === form.projectId)?.name ?? form.projectId
				)
			: 'All projects'
	);

	const selectedSessionLabel = $derived(
		form.sessionId ? truncateLabel(form.sessionId, 32) : 'All sessions'
	);
</script>

<Card>
	<CardContent>
		<form class="space-y-4" onsubmit={onSubmit}>
			<div class="grid gap-3 lg:grid-cols-[minmax(0,1.8fr)_minmax(0,1.1fr)_minmax(0,1fr)]">
				<div class="space-y-2">
				<label for="search" class="text-xs text-muted-foreground">Search</label>
				<div class="relative">
					<Search class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						id="search"
						type="search"
						class="pl-9"
						placeholder="model, tool, prompt, output, event..."
						bind:value={form.query}
					/>
				</div>
			</div>

				<div class="space-y-2">
					<label for="action" class="text-xs text-muted-foreground">Action</label>
					<Input id="action" placeholder="tool.called" bind:value={form.action} />
				</div>

				<div class="flex items-end">
					<Button type="submit" class="w-full">Run query</Button>
				</div>
			</div>

			<div class="grid gap-3 md:grid-cols-2 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,1.2fr)_minmax(0,0.7fr)]">
				<div class="space-y-2">
					<p class="text-xs text-muted-foreground">Project</p>
					<Select
						type="single"
						bind:value={form.projectId}
						onValueChange={onProjectChange}
					>
						<SelectTrigger class="w-full max-w-full">
							<span class="block truncate text-left">{selectedProjectLabel}</span>
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="">All projects</SelectItem>
							{#each projects as project}
								<SelectItem value={project.id}>{project.name ?? project.id}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>

				<div class="space-y-2">
					<p class="text-xs text-muted-foreground">Session</p>
					<Select
						type="single"
						bind:value={form.sessionId}
						onValueChange={onSessionChange}
					>
						<SelectTrigger class="w-full max-w-full">
							<span class="block truncate text-left">{selectedSessionLabel}</span>
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="">All sessions</SelectItem>
							{#each sessions as session}
								<SelectItem value={session.id}>{session.provider} / {session.model}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>

				<div class="space-y-2 md:col-span-2 xl:col-span-1">
					<p class="text-xs text-muted-foreground">Events shown</p>
					<Select
						type="single"
						bind:value={form.eventLimit}
						onValueChange={onEventLimitChange}
					>
						<SelectTrigger class="w-full">{form.eventLimit}</SelectTrigger>
						<SelectContent>
							{#each EVENT_LIMIT_OPTIONS as option}
								<SelectItem value={option}>{option}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>
			</div>
		</form>
	</CardContent>
</Card>
