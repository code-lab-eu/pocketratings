<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteLocation } from '$lib/api';
	import ManageListRow from '$lib/ManageListRow.svelte';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';
	import type { Location } from '$lib/types';

	let { data } = $props();
	let locations = $derived(data.locations);
	let error = $derived(data.error);
	let deletingId = $state<string | null>(null);

	async function handleDelete(loc: Location) {
		if (deletingId) return;
		if (!confirm(`Delete location "${loc.name}"?`)) return;
		deletingId = loc.id;
		try {
			await deleteLocation(loc.id);
			await goto(resolve('/manage/locations'), { invalidateAll: true });
		} catch (e) {
			alert(e instanceof Error ? e.message : String(e));
		} finally {
			deletingId = null;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/manage')}
			class="pr-link-muted"
			>← Manage</a
		>
	</nav>
	<PageHeading>Locations</PageHeading>
	<Button
		variant="primary"
		href={resolve('/manage/locations/new')}
		class="mb-4 inline-block"
	>
		New location
	</Button>

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if locations.length === 0}
		<p class="pr-text-muted">No locations yet.</p>
	{:else}
		<ul class="space-y-2">
			{#each locations as location (location.id)}
				<ManageListRow
					label={location.name}
					editHref={resolve('/manage/locations/[id]', { id: location.id })}
					deleteLabel={location.name}
					onDelete={() => handleDelete(location)}
					deleting={deletingId === location.id}
				/>
			{/each}
		</ul>
	{/if}
</main>
