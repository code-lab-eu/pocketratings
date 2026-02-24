<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteProduct, updateProduct } from '$lib/api';
	import { flattenCategories } from '$lib/categories';
	import CategorySelect from '$lib/CategorySelect.svelte';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';

	let { data } = $props();
	let product = $derived(data.product);
	let categories = $derived(data.categories);
	let categoryOptions = $derived(flattenCategories(categories));
	let error = $derived(data.error);
	let notFound = $derived(data.notFound ?? false);

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
		<a
			href={resolve('/manage/products')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Products</a
		>
	</nav>

	{#if notFound}
		<p class="pr-text-muted">Product not found.</p>
		<p class="mt-2">
			<a
				href={resolve('/manage/products')}
				class="pr-link-inline"
				>Back to products</a
			>
		</p>
	{:else if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if product}
		<PageHeading>Edit product</PageHeading>

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
			<div>
				<label for="brand" class="mb-1 block pr-text-label">Brand</label>
				<input
					id="brand"
					type="text"
					bind:value={brand}
					class="pr-input"
					autocomplete="off"
				/>
			</div>
			<CategorySelect
				options={categoryOptions}
				bind:value={categoryId}
				id="category"
				label="Category"
				placeholder=""
				required
			/>
			<div class="flex flex-wrap gap-2">
				<Button type="submit" disabled={submitting} variant="primary">
					{submitting ? 'Saving…' : 'Save'}
				</Button>
				<Button variant="secondary" href={resolve('/manage/products')}>
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
		<p class="pr-text-muted">Product not found.</p>
	{/if}
</main>
