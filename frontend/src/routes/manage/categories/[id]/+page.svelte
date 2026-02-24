<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteCategory, updateCategory } from '$lib/api';
	import { flattenCategories } from '$lib/categories';
	import CategorySelect from '$lib/CategorySelect.svelte';

	let { data } = $props();
	let category = $derived(data.category);
	let categories = $derived(data.categories);
	let parentOptions = $derived(
		category ? flattenCategories(categories).filter(({ category: c }) => c.id !== category.id) : []
	);
	let error = $derived(data.error);
	let notFound = $derived(data.notFound ?? false);

	let name = $state('');
	let parentId = $state('');
	let submitting = $state(false);
	let formError = $state<string | null>(null);

	// $effect() runs a side effect when any reactive dependency (e.g. category) changes.
	// Here we sync the form fields (name, parentId) whenever the loaded category changes
	// (e.g. after navigation to this page or after load re-runs).
	$effect(() => {
		if (category) {
			name = category.name;
			parentId = category.parent_id ?? '';
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!category) return;
		formError = null;
		const n = name.trim();
		if (!n) {
			formError = 'Name is required';
			return;
		}
		submitting = true;
		try {
			await updateCategory(category.id, { name: n, parent_id: parentId || null });
			await goto(resolve('/manage/categories'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}

	async function handleDelete() {
		if (!category) return;
		if (!confirm(`Delete category "${category.name}"?`)) return;
		try {
			await deleteCategory(category.id);
			await goto(resolve('/manage/categories'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/manage/categories')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Categories</a
		>
	</nav>

	{#if notFound}
		<p class="text-gray-600 dark:text-gray-200">Category not found.</p>
		<p class="mt-2">
			<a
				href={resolve('/manage/categories')}
				class="text-gray-900 underline hover:no-underline dark:text-gray-50"
				>Back to categories</a
			>
		</p>
	{:else if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if category}
		<h1 class="mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50">Edit category</h1>

		<form onsubmit={handleSubmit} class="space-y-4">
			{#if formError}
				<p class="text-red-600 dark:text-red-300">{formError}</p>
			{/if}
			<div>
				<label
					for="name"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Name</label
				>
				<input
					id="name"
					type="text"
					bind:value={name}
					required
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
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
			<div class="flex flex-wrap gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50 dark:border-gray-600 dark:bg-gray-50 dark:text-gray-900 dark:hover:bg-gray-200"
				>
					{submitting ? 'Saving…' : 'Save'}
				</button>
				<a
					href={resolve('/manage/categories')}
					class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50 dark:hover:bg-gray-700"
					>
					Cancel
				</a>
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
		<p class="text-gray-600 dark:text-gray-200">Category not found.</p>
	{/if}
</main>
