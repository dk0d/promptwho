<script lang="ts">
	import { Activity, FolderTree, MessagesSquare } from "@lucide/svelte";
	import logoUrl from "../../../../../assets/promptwho-logo.png";
	import LightSwitch from "$lib/elements/light-switch.svelte";
	import {
		Card,
		CardDescription,
		CardHeader,
		CardTitle,
	} from "$lib/shadcn/components/ui/card";

	let {
		baseUrl,
		error,
		projectCount,
		sessionCount,
		eventCount,
	}: {
		baseUrl: string;
		error: string | null;
		projectCount: number;
		sessionCount: number;
		eventCount: number;
	} = $props();

	const stats = $derived([
		{ label: "Projects", value: projectCount, Icon: FolderTree },
		{ label: "Sessions", value: sessionCount, Icon: MessagesSquare },
		{ label: "Events", value: eventCount, Icon: Activity },
	]);
</script>

<Card
	class="overflow-hidden border-primary/15 bg-card/90 shadow-sm backdrop-blur"
>
	<div
		class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,oklch(0.93_0.06_24/.7),transparent_34%),radial-gradient(circle_at_bottom_right,oklch(0.92_0.03_320/.55),transparent_28%)] dark:bg-[radial-gradient(circle_at_top_left,oklch(0.32_0.08_24/.35),transparent_34%),radial-gradient(circle_at_bottom_right,oklch(0.36_0.04_320/.22),transparent_28%)]"
	></div>
	<CardHeader class="relative gap-6 p-5 sm:p-7">
		<div class="flex items-start justify-end gap-4">
			<LightSwitch />
		</div>

		<div class="grid gap-6 xl:grid-cols-[minmax(0,1fr)_20rem] xl:items-start">
			<div class="flex flex-col gap-5 sm:flex-row sm:items-start">
				<div
					class="flex h-24 w-24 shrink-0 items-center justify-center rounded-3xl border border-primary/15 bg-background/75 shadow-sm backdrop-blur sm:h-28 sm:w-28 overflow-hidden"
				>
					<img class="max-h-full w-auto" src={logoUrl} alt="PromptWho logo" />
				</div>

				<div class="space-y-3">
					<div class="space-y-2">
						<p
							class="text-sm font-medium uppercase tracking-[0.22em] text-primary/75"
						>
							Query your AI-assisted work
						</p>
						<CardTitle class="text-3xl leading-tight sm:text-4xl">
							PromptWho Dashboard
						</CardTitle>
						<CardDescription class="max-w-3xl text-sm leading-6 sm:text-base">
							Search prompts and outputs, inspect session timelines, and explore
							the raw event stream from <code>{baseUrl}</code>.
						</CardDescription>
					</div>

					<div
						class="flex flex-wrap items-center gap-3 text-sm text-muted-foreground"
					>
						<div
							class="rounded-full border bg-background/70 px-3 py-1.5 backdrop-blur"
						>
							Rust server + local dashboard
						</div>
						<div
							class="rounded-full border bg-background/70 px-3 py-1.5 backdrop-blur"
						>
							MsgPack ingest + query views
						</div>
					</div>

					{#if error}
						<p class="text-sm text-destructive">
							The latest dashboard refresh failed.
						</p>
					{/if}
				</div>
			</div>

			<div class="grid gap-3 sm:grid-cols-3 xl:grid-cols-1">
				{#each stats as stat}
					<div
						class="rounded-2xl border bg-background/80 px-4 py-3 shadow-sm backdrop-blur"
					>
						<div class="flex items-center justify-between gap-3">
							<div>
								<p
									class="text-xs uppercase tracking-[0.18em] text-muted-foreground"
								>
									{stat.label}
								</p>
								<p class="mt-1 text-2xl font-semibold">{stat.value}</p>
							</div>
							<div class="rounded-xl border bg-primary/8 p-2.5 text-primary">
								<stat.Icon class="size-4" />
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</CardHeader>
</Card>
