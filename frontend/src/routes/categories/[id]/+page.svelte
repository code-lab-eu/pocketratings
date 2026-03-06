<script lang="ts">
	import { resolve } from '$app/paths';
	import ProductList from '$lib/ProductList.svelte';
	import SearchForm from '$lib/SearchForm.svelte';

	let { data } = $props();
	let category = $derived(data.category);
	let query = $derived(data.query ?? '');
	let allChildren = $derived(category?.children ?? []);
	let childCategories = $derived(
		query.trim() === ''
			? allChildren
			: allChildren.filter((child) =>
					child.name.toLowerCase().includes(query.toLowerCase())
				)
	);
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
	<nav class="mb-4" aria-label="Breadcrumb">
		{#if category}
			<ol class="flex flex-wrap items-center gap-x-1 text-sm pr-text-muted">
				<li>
					<a href={resolve('/')} class="pr-link-muted">Home</a>
				</li>
				{#each [...(category.ancestors ?? [])].reverse() as ancestor (ancestor.id)}
					<li class="flex items-center gap-x-1">
						<span aria-hidden="true">/</span>
						<a
							href={resolve(`/categories/${ancestor.id}`)}
							class="pr-link-muted"
						>
							{ancestor.name}
						</a>
					</li>
				{/each}
				<li class="flex items-center gap-x-1" aria-current="page">
					<span aria-hidden="true">/</span>
					<span>{category.name}</span>
				</li>
			</ol>
		{:else}
			<a href={resolve('/')} class="pr-link-muted">← Home</a>
		{/if}
	</nav>

	{#if notFound}
		<p class="pr-text-muted">Category not found.</p>
		<p class="mt-2">
			<a
				href={resolve('/')}
				class="pr-link-inline"
				>Back to home</a
			>
		</p>
	{:else if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if category}
		<SearchForm
			actionUrl={resolve(`/categories/${category.id}`)}
			query={query}
			placeholder={'Search in category "' + category.name + '"'}
		/>
		<h1 class="pr-heading-page">{category.name}</h1>
		<p class="mb-4">
			<!-- eslint-disable svelte/no-navigation-without-resolve -- href is resolve() + query string; rule only accepts direct resolve() -->
			<a
				href={`${resolve('/manage/products/new')}?category_id=${encodeURIComponent(category.id)}`}
				class="pr-link-inline"
			>
				Add product
			</a>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</p>
		{#if childCategories.length > 0}
			<ul class="mb-6 space-y-2">
				{#each childCategories as child (child.id)}
					<li>
						<a
							href={resolve(`/categories/${child.id}`)}
							class="pr-card block"
						>
							{child.name}
						</a>
					</li>
				{/each}
			</ul>
		{:else if query.trim() !== ''}
			<p class="pr-text-muted mb-6">No categories match.</p>
		{/if}
		{#if items.length === 0}
			<p class="pr-text-muted">No products in this category.</p>
		{:else}
			<ProductList items={items} />
		{/if}
	{:else}
		<p class="pr-text-muted">Category not found.</p>
	{/if}
</main>
