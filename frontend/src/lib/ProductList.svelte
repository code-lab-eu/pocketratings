<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Product } from '$lib/types';

	interface ProductWithReview {
		product: Product;
		rating?: number;
		text?: string | null;
	}

	let { items = [] }: { items: ProductWithReview[] } = $props();
</script>

<ul class="space-y-2">
	{#each items as { product, rating, text } (product.id)}
		<li class="min-w-0">
			<a
				href={resolve(`/products/${product.id}`)}
				class="block min-h-[44px] rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50 break-words"
			>
				<span class="font-medium">{product.name}</span>
				{#if product.brand}
					<span class="text-gray-600"> — {product.brand}</span>
				{/if}
				{#if rating != null}
					<span class="mt-1 block text-sm text-gray-600" aria-label="Your rating">
						Rating: {rating}/5
					</span>
				{/if}
				{#if text}
					<p class="mt-1 line-clamp-1 text-sm text-gray-500">{text}</p>
				{/if}
			</a>
		</li>
	{/each}
</ul>
