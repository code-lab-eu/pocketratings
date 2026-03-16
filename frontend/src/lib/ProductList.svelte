<script lang="ts">
  import { resolve } from '$app/paths';
  import PriceDisplay from '$lib/PriceDisplay.svelte';
  import StarRating from '$lib/StarRating.svelte';
  import type { Product } from '$lib/types';

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
        {#if product.review_score != null || (product.price != null && product.price !== '')}
          <div class="mt-1 flex min-h-[1.25rem] items-center justify-between gap-2">
            {#if product.review_score != null}
              <StarRating score={product.review_score} />
            {:else}
              <span></span>
            {/if}
            {#if product.price != null && product.price !== ''}
              <PriceDisplay amount={product.price} />
            {/if}
          </div>
        {/if}
      </a>
    </li>
  {/each}
</ul>
