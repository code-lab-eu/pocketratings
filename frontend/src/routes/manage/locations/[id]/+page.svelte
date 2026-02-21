<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteLocation, updateLocation } from '$lib/api';

	let { data } = $props();
	let location = $derived(data.location);
	let error = $derived(data.error);

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
		<a href={resolve('/manage/locations')} class="text-gray-600 hover:text-gray-900">← Locations</a>
	</nav>

	{#if error}
		<p class="text-red-600">{error}</p>
	{:else if location}
		<h1 class="mb-4 text-2xl font-semibold text-gray-900">Edit location</h1>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if formError}
				<p class="text-red-600">{formError}</p>
			{/if}
			<div>
				<label for="name" class="mb-1 block text-sm font-medium text-gray-700">Name</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					required
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900"
					autocomplete="off"
				/>
			</div>
			<div class="flex flex-wrap gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50"
				>
					{submitting ? 'Saving…' : 'Save'}
				</button>
				<a href={resolve('/manage/locations')} class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50">
					Cancel
				</a>
				<button
					type="button"
					onclick={handleDelete}
					class="rounded-lg border border-red-300 px-4 py-2 text-red-700 hover:bg-red-50"
				>
					Delete
				</button>
			</div>
		</form>
	{:else}
		<p class="text-gray-600">Location not found.</p>
	{/if}
</main>
