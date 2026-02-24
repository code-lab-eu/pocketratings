<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteLocation, updateLocation } from '$lib/api';

	let { data } = $props();
	let location = $derived(data.location);
	let error = $derived(data.error);
	let notFound = $derived(data.notFound ?? false);

	let name = $state('');
	let submitting = $state(false);
	let formError = $state<string | null>(null);

	$effect(() => {
		if (location) {
			name = location.name;
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!location) return;
		formError = null;
		const n = name.trim();
		if (!n) {
			formError = 'Name is required';
			return;
		}
		submitting = true;
		try {
			await updateLocation(location.id, { name: n });
			await goto(resolve('/manage/locations'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}

	async function handleDelete() {
		if (!location) return;
		if (!confirm(`Delete location "${location.name}"?`)) return;
		try {
			await deleteLocation(location.id);
			await goto(resolve('/manage/locations'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/manage/locations')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Locations</a
		>
	</nav>

	{#if notFound}
		<p class="pr-text-muted">Location not found.</p>
		<p class="mt-2">
			<a
				href={resolve('/manage/locations')}
				class="pr-link-inline"
				>Back to locations</a
			>
		</p>
	{:else if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if location}
		<PageHeading>Edit location</PageHeading>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if formError}
				<p class="text-red-600 dark:text-red-300">{formError}</p>
			{/if}
			<div>
				<label for="name" class="mb-1 block pr-text-label">Name</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					required
					class="pr-input"
					autocomplete="off"
				/>
			</div>
			<div class="flex flex-wrap gap-2">
				<Button type="submit" disabled={submitting} variant="primary">
					{submitting ? 'Saving…' : 'Save'}
				</Button>
				<Button variant="secondary" href={resolve('/manage/locations')}>
					Cancel
				</Button>
				<button
					type="button"
					onclick={handleDelete}
					class="rounded-lg border border-red-300 px-4 py-2 text-red-700 hover:bg-red-50 dark:border-red-500 dark:bg-transparent dark:text-red-300 dark:hover:bg-red-950"
				>
					Delete
				</button>
			</div>
		</form>
	{:else}
		<p class="pr-text-muted">Location not found.</p>
	{/if}
</main>
