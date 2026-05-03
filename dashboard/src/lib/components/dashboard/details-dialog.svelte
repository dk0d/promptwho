<script lang="ts">
	import { Button } from "$lib/shadcn/components/ui/button";
	import * as Dialog from "$lib/shadcn/components/ui/dialog";
	import {
		ScrollArea,
		ScrollAreaScrollbar,
	} from "$lib/shadcn/components/ui/scroll-area";
	import { EllipsisIcon } from "@lucide/svelte";

	let {
		title,
		description = "",
		data,
	}: {
		title: string;
		description?: string;
		data: object;
	} = $props();

	let open = $state(false);

	const entries = $derived(Object.entries(data as Record<string, unknown>));

	function formatLabel(key: string) {
		return key.replaceAll("_", " ");
	}

	function formatValue(value: unknown) {
		if (value === null || value === undefined) return "None";
		if (typeof value === "object") return JSON.stringify(value, null, 2);
		return String(value);
	}

	function isBlockValue(value: unknown) {
		return (
			typeof value === "object" ||
			(typeof value === "string" && value.includes("\n"))
		);
	}
</script>

<Button type="button" size="icon" variant="ghost" onclick={() => (open = true)}
	><EllipsisIcon /></Button
>

<Dialog.Root bind:open>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>{title}</Dialog.Title>
			<Dialog.Description>{description}</Dialog.Description>
		</Dialog.Header>

		<ScrollArea class="max-h-[70vh] pr-4">
			<div class="space-y-4 p-4">
				{#each entries as [key, value]}
					<div class="space-y-1.5">
						<p
							class="text-xs font-medium tracking-wide text-muted-foreground uppercase"
						>
							{formatLabel(key)}
						</p>
						{#if isBlockValue(value)}
							<pre
								class="overflow-x-auto rounded-md bg-muted p-3 text-xs whitespace-pre-wrap break-words">{formatValue(
									value,
								)}</pre>
						{:else}
							<p class="text-sm break-words">{formatValue(value)}</p>
						{/if}
					</div>
				{/each}
			</div>
			<ScrollAreaScrollbar orientation="vertical" />
		</ScrollArea>
	</Dialog.Content>
</Dialog.Root>
