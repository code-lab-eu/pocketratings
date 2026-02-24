<script lang="ts">
	/* eslint-disable svelte/no-navigation-without-resolve -- Add review/purchase links need query params; path uses resolve() */
	import { resolve } from '$app/paths';

	let { data } = $props();

	let product = $derived(data.product);
	let reviews = $derived(data.reviews);
	let purchases = $derived(data.purchases);
	let category = $derived(data.category);
	let locationNames = $derived(data.locationNames);
	let error = $derived(data.error);
	let notFound = $derived(data.notFound ?? false);

	function formatDate(unixSeconds: number): string {
		return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function locationName(locationId: string): string {
		const map = locationNames as Record<string, string>;
		return map[locationId] ?? locationId;
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<p class="mb-4">
		<a href={resolve('/')} class="text-gray-600 hover:text-gray-900">← Home</a>
	</p>

	{#if notFound}
		<p class="text-gray-600">Product not found.</p>
		<p class="mt-2">
			<a href={resolve('/')} class="text-gray-900 underline hover:no-underline">Back to home</a>
		</p>
	{:else if error}
		<p class="text-red-600">{error}</p>
	{:else if !product}
		<p class="text-gray-600">Product not found.</p>
	{:else}
		<article class="min-w-0">
			<header class="mb-6">
				<h1 class="break-words text-xl font-semibold text-gray-900">{product.name}</h1>
				{#if product.brand}
					<p class="text-gray-600">{product.brand}</p>
				{/if}
				<p class="mt-1 text-gray-600">
					Category:
					<a
						href={resolve('/categories/[id]', { id: product.category_id })}
						class="text-gray-900 underline hover:no-underline"
					>
						{category?.name ?? 'Category'}
					</a>
				</p>
			</header>

			<section class="mb-6" aria-labelledby="reviews-heading">
				<h2 id="reviews-heading" class="mb-3 text-lg font-medium text-gray-900">Reviews</h2>
				{#if reviews.length === 0}
					<p class="text-gray-600">No reviews yet.</p>
				{:else}
					<ul class="space-y-3">
						{#each reviews as review (review.id)}
							<li class="rounded-lg border border-gray-200 bg-white p-4">
								<p class="font-medium text-gray-900">Rating: {review.rating}/5</p>
								{#if review.text}
									<p class="mt-1 text-gray-700">{review.text}</p>
								{/if}
								<p class="mt-1 text-sm text-gray-500">
									{formatDate(review.updated_at)}
								</p>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="mb-6" aria-labelledby="purchase-history-heading">
				<h2 id="purchase-history-heading" class="mb-3 text-lg font-medium text-gray-900">Purchase history</h2>
				{#if purchases.length === 0}
					<p class="text-gray-600">No purchases recorded.</p>
				{:else}
					<ul class="space-y-2">
						{#each purchases as purchase (purchase.id)}
							<li class="flex flex-wrap gap-x-4 gap-y-1 text-gray-700">
								<span>{formatDate(purchase.purchased_at)}</span>
								<span>{locationName(purchase.location_id)}</span>
								<span>{purchase.price} €</span>
							</li>
						{/each}
					</ul>
				{/if}
			</section>

			<section class="border-t border-gray-200 pt-4" aria-label="Actions">
				<p class="text-gray-600">
					<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolve() + query string; rule only accepts direct resolve() -->
					<a href={`${resolve('/manage/reviews/add')}?product_id=${product.id}`}
						class="text-gray-900 underline hover:no-underline"
					>
						Add review
					</a>
					<span class="mx-2">·</span>
					<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolve() + query string; rule only accepts direct resolve() -->
					<a href={`${resolve('/manage/purchases/add')}?product_id=${product.id}`}
						class="text-gray-900 underline hover:no-underline"
					>
						Add purchase
					</a>
				</p>
			</section>
		</article>
	{/if}
</main>
