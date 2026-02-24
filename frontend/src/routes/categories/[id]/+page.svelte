<script lang="ts">
	import { resolve } from '$app/paths';
	import ProductList from '$lib/ProductList.svelte';

	let { data } = $props();
	let category = $derived(data.category);
	let childCategories = $derived(data.childCategories);
	let items = $derived(data.items);
	let error = $derived(data.error);
	let notFound = $derived(data.notFound ?? false);
</script>

<svelte:head>
	{#if category}
		<title>{category.name} — Pocket Ratings</title>
	{/if}
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Home</a
		>
	</nav>

	{#if notFound}
		<p class="text-gray-600 dark:text-gray-200">Category not found.</p>
		<p class="mt-2">
			<a
				href={resolve('/')}
				class="text-gray-900 underline hover:no-underline dark:text-gray-50"
				>Back to home</a
			>
		</p>
	{:else if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if category}
		<h1 class="mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50">{category.name}</h1>
		{#if childCategories.length > 0}
			<ul class="mb-6 space-y-2">
				{#each childCategories as child (child.id)}
					<li>
						<a
							href={resolve(`/categories/${child.id}`)}
							class="block rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-50 dark:hover:bg-gray-700"
						>
							{child.name}
						</a>
					</li>
				{/each}
			</ul>
		{/if}
		{#if items.length === 0}
			<p class="text-gray-600 dark:text-gray-200">No products in this category.</p>
		{:else}
			<ProductList items={items} />
		{/if}
	{:else}
		<p class="text-gray-600 dark:text-gray-200">Category not found.</p>
	{/if}
</main>
