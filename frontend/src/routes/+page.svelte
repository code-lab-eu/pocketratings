<script lang="ts">
	import { resolve } from '$app/paths';
	import CategoryLinkList from '$lib/CategoryLinkList.svelte';
	import ProductList from '$lib/ProductList.svelte';
	import SearchForm from '$lib/SearchForm.svelte';

	let { data } = $props();
	let categories = $derived(data.categories);
	let items = $derived(data.items);
	let query = $derived(data.query);
	let error = $derived(data.error);
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<SearchForm actionUrl={resolve('/')} query={query} />

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else}
		<section class="mb-8" aria-labelledby="categories-heading">
			<h2 id="categories-heading" class="pr-heading-section">
				Categories
			</h2>
			{#if categories.length === 0}
				<p class="pr-text-muted">No categories match.</p>
			{:else}
				<CategoryLinkList items={categories} basePath="categories" />
			{/if}
		</section>

		<section aria-labelledby="products-heading">
			<h2 id="products-heading" class="pr-heading-section">
				Products
			</h2>
			{#if items.length === 0}
				<p class="pr-text-muted">No products match.</p>
			{:else}
				<ProductList {items} />
			{/if}
		</section>
	{/if}
</main>
