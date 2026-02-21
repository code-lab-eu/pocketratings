<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createCategory } from '$lib/api';

	let { data } = $props();
	let categories = $derived(data.categories);
	let loadError = $derived(data.error);

	let name = $state('');
	let parentId = $state<string>('');
	let submitting = $state(false);
	let error = $state<string | null>(null);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;
		const n = name.trim();
		if (!n) {
			error = 'Name is required';
			return;
		}
		submitting = true;
		try {
			await createCategory({ name: n, parent_id: parentId || null });
			await goto(resolve('/manage/categories'), { invalidateAll: true });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a href={resolve('/manage/categories')} class="text-gray-600 hover:text-gray-900">← Categories</a>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900">New category</h1>

	{#if loadError}
		<p class="text-red-600">{loadError}</p>
	{:else}
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if error}
			<p class="text-red-600">{error}</p>
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
		<div>
			<label for="parent" class="mb-1 block text-sm font-medium text-gray-700">Parent (optional)</label>
			<select
				id="parent"
				bind:value={parentId}
				class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900"
			>
				<option value="">None</option>
				{#each categories as cat (cat.id)}
					<option value={cat.id}>{cat.name}</option>
				{/each}
			</select>
		</div>
		<div class="flex gap-2">
			<button
				type="submit"
				disabled={submitting}
				class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50"
			>
				{submitting ? 'Creating…' : 'Create'}
			</button>
			<a href={resolve('/manage/categories')} class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50">
				Cancel
			</a>
		</div>
	</form>
	{/if}
</main>
