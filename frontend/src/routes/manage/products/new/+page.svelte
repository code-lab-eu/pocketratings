<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createProduct } from '$lib/api';
	import { flattenCategories } from '$lib/categories';
	import CategorySelect from '$lib/CategorySelect.svelte';

	let { data } = $props();
	let categories = $derived(data.categories);
	let categoryOptions = $derived(flattenCategories(categories));
	let loadError = $derived(data.error);

	let name = $state('');
	let brand = $state('');
	let categoryId = $state('');
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
		if (!categoryId) {
			error = 'Category is required';
			return;
		}
		submitting = true;
		try {
			await createProduct({ name: n, brand: brand.trim(), category_id: categoryId });
			await goto(resolve('/manage/products'), { invalidateAll: true });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a href={resolve('/manage/products')} class="text-gray-600 hover:text-gray-900">← Products</a>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900">New product</h1>

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
				<label for="brand" class="mb-1 block text-sm font-medium text-gray-700">Brand</label>
				<input
					id="brand"
					type="text"
					bind:value={brand}
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900"
					autocomplete="off"
				/>
			</div>
			<CategorySelect
				options={categoryOptions}
				bind:value={categoryId}
				id="category"
				label="Category"
				placeholder="Select category"
				required
			/>
			<div class="flex gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50"
				>
					{submitting ? 'Creating…' : 'Create'}
				</button>
				<a href={resolve('/manage/products')} class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50">
					Cancel
				</a>
			</div>
		</form>
	{/if}
</main>
