<script lang="ts">
  import { resolve } from '$app/paths';
  import type { Product } from '$lib/types';
  import { formatRating } from '$lib/utils/formatters';

  /** Item for ProductList: product (with optional review_score and price from API). */
  interface ProductListItem {
    product: Product;
  }

  let { items = [] }: { items: ProductListItem[] } = $props();
</script>

<ul class="space-y-2">
  {#each items as { product } (product.id)}
    <li class="min-w-0">
      <a
        href={resolve(`/products/${product.id}`)}
        class="pr-card block min-h-[44px] break-words"
      >
        <span class="font-medium">{product.name}</span>
        {#if product.brand}
          <span class="pr-text-muted"> — {product.brand}</span>
        {/if}
        {#if product.review_score != null}
          <span class="mt-1 block text-sm pr-rating" aria-label="Rating">
            Rating: {formatRating(product.review_score)}/5
          </span>
        {/if}
        {#if product.price != null && product.price !== ''}
          <span class="mt-1 block text-sm pr-text-muted">Price: {product.price}</span>
        {/if}
      </a>
    </li>
  {/each}
</ul>
