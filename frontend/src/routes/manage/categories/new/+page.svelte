<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createCategory } from '$lib/api';
	import { flattenCategories } from '$lib/categories';
	import CategorySelect from '$lib/CategorySelect.svelte';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';

	let { data } = $props();
	let categories = $derived(data.categories);
	let parentOptions = $derived(flattenCategories(categories));
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
		<a
			href={resolve('/manage/categories')}
			class="pr-link-muted"
			>← Categories</a
		>
	</nav>
	<PageHeading>New category</PageHeading>

	{#if loadError}
		<p class="text-red-600 dark:text-red-300">{loadError}</p>
	{:else}
	<form onsubmit={handleSubmit} class="space-y-4">
		{#if error}
			<p class="text-red-600 dark:text-red-300">{error}</p>
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
		<CategorySelect
			options={parentOptions}
			bind:value={parentId}
			id="parent"
			label="Parent (optional)"
			placeholder="None"
		/>
		<div class="flex gap-2">
			<Button type="submit" disabled={submitting} variant="primary">
				{submitting ? 'Creating…' : 'Create'}
			</Button>
			<Button variant="secondary" href={resolve('/manage/categories')}>
				Cancel
			</Button>
		</div>
	</form>
	{/if}
</main>
