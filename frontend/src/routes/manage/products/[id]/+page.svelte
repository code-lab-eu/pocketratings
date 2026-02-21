<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteProduct, updateProduct } from '$lib/api';

	let { data } = $props();
	let product = $derived(data.product);
	let categories = $derived(data.categories);
	let error = $derived(data.error);

	let name = $state('');
	let brand = $state('');
	let categoryId = $state('');
	let submitting = $state(false);
	let formError = $state<string | null>(null);

	$effect(() => {
		if (product) {
			name = product.name;
			brand = product.brand;
			categoryId = product.category_id;
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!product) return;
		formError = null;
		const n = name.trim();
		if (!n) {
			formError = 'Name is required';
			return;
		}
		if (!categoryId) {
			formError = 'Category is required';
			return;
		}
		submitting = true;
		try {
			await updateProduct(product.id, { name: n, brand: brand.trim(), category_id: categoryId });
			await goto(resolve('/manage/products'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}

	async function handleDelete() {
		if (!product) return;
		if (!confirm(`Delete product "${product.name}"?`)) return;
		try {
			await deleteProduct(product.id);
			await goto(resolve('/manage/products'), { invalidateAll: true });
		} catch (e) {
			formError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a href={resolve('/manage/products')} class="text-gray-600 hover:text-gray-900">← Products</a>
	</nav>

	{#if error}
		<p class="text-red-600">{error}</p>
	{:else if product}
		<h1 class="mb-4 text-2xl font-semibold text-gray-900">Edit product</h1>

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
			<div>
				<label for="category" class="mb-1 block text-sm font-medium text-gray-700">Category</label>
				<select
					id="category"
					bind:value={categoryId}
					required
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900"
				>
					{#each categories as cat (cat.id)}
						<option value={cat.id}>{cat.name}</option>
					{/each}
				</select>
			</div>
			<div class="flex flex-wrap gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50"
				>
					{submitting ? 'Saving…' : 'Save'}
				</button>
				<a href={resolve('/manage/products')} class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50">
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
		<p class="text-gray-600">Product not found.</p>
	{/if}
</main>
