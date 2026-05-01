<script lang="ts">
	import "./layout.css";
	import favicon from "$lib/assets/favicon.svg";
	import { ModeWatcher } from "mode-watcher";
	import LightSwitch from "$lib/elements/light-switch.svelte";

	let { children } = $props();
</script>

<ModeWatcher />

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="flex justify-end items-center p-8">
	<LightSwitch />
</div>

{@render children()}
