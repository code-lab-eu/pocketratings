<script lang="ts">
	import { resolve } from '$app/paths';
	import { listProducts, listReviews } from '$lib/api';
	import CategoryLinkList from '$lib/CategoryLinkList.svelte';
	import ProductList from '$lib/ProductList.svelte';
	import SearchForm from '$lib/SearchForm.svelte';
	import type { Category, Product, Review } from '$lib/types';
	import type { ProductWithReview } from './+page';

	type CategoryWithDepth = { category: Category; depth: number };

	let { data } = $props();

	let displayedCategories = $state<CategoryWithDepth[]>([]);
	let displayedItems = $state<ProductWithReview[]>([]);
	let displayedError = $state<string | null>(null);
	let searchQuery = $state('');

	$effect(() => {
		displayedCategories = data.categories;
		displayedItems = data.items;
		displayedError = data.error;
		searchQuery = data.query;
	});

	function mergeProductsWithReviews(
		products: Product[],
		reviews: Review[]
	): ProductWithReview[] {
		const reviewByProductId: Record<string, Review> = {};
		for (const r of reviews) {
			const existing = reviewByProductId[r.product.id];
			if (!existing || r.updated_at > existing.updated_at) {
				reviewByProductId[r.product.id] = r;
			}
		}
		return products.map((product) => {
			const review = reviewByProductId[product.id];
			return {
				product,
				rating: review != null ? review.rating : undefined,
				text: review?.text ?? undefined
			};
		});
	}

	async function onQueryChange(q: string) {
		const path = resolve('/');
		const url = q ? `${path}?q=${encodeURIComponent(q)}` : path;
		if (typeof history !== 'undefined') {
			history.replaceState(null, '', url);
		}

		if (q === '') {
			displayedCategories = data.categories;
			displayedItems = data.items;
			displayedError = data.error;
			searchQuery = '';
			return;
		}

		searchQuery = q;
		try {
			const [products, reviews] = await Promise.all([
				listProducts({ q }),
				listReviews()
			]);
			const full = data.fullCategories ?? data.categories;
			displayedCategories = full.filter(({ category }) =>
				category.name.toLowerCase().includes(q.toLowerCase())
			);
			displayedItems = mergeProductsWithReviews(products, reviews);
			displayedError = null;
		} catch (e) {
			displayedError = e instanceof Error ? e.message : String(e);
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<SearchForm
		actionUrl={resolve('/')}
		query={searchQuery}
		onQueryChange={onQueryChange}
	/>

	{#if displayedError}
		<p class="text-red-600 dark:text-red-300">{displayedError}</p>
	{:else}
		<section class="mb-8" aria-labelledby="categories-heading">
			<h2 id="categories-heading" class="pr-heading-section">
				Categories
			</h2>
			{#if displayedCategories.length === 0}
				<p class="pr-text-muted">No categories match.</p>
			{:else}
				<CategoryLinkList items={displayedCategories} basePath="categories" />
			{/if}
		</section>

		<section aria-labelledby="products-heading">
			<h2 id="products-heading" class="pr-heading-section">
				Products
			</h2>
			{#if displayedItems.length === 0}
				<p class="pr-text-muted">No products match.</p>
			{:else}
				<ProductList items={displayedItems} />
			{/if}
		</section>
	{/if}
</main>
