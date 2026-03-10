<script lang="ts">
  import { resolve } from '$app/paths';
  import BackLink from '$lib/BackLink.svelte';
  import FormError from '$lib/FormError.svelte';
  import NotFoundMessage from '$lib/NotFoundMessage.svelte';

  let { data } = $props();

  let product = $derived(data.product);
  let reviews = $derived(data.reviews);
  let purchases = $derived(data.purchases);
  let error = $derived(data.error);
  let notFound = $derived(data.notFound ?? false);

  function formatDate(unixSeconds: number): string {
    return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/')} label="Home" />

  {#if notFound}
    <NotFoundMessage
      message="Product not found."
      backHref={resolve('/')}
      backLabel="Back to home"
    />
  {:else if error}
    <FormError message={error} />
  {:else if !product}
    <NotFoundMessage
      message="Product not found."
      backHref={resolve('/')}
      backLabel="Back to home"
    />
  {:else}
    <article class="min-w-0">
      <header class="mb-6">
        <h1 class="break-words pr-text-body text-xl font-semibold">
          {product.name}
        </h1>
        {#if product.brand}
          <p class="pr-text-muted">{product.brand}</p>
        {/if}
        <p class="mt-1 pr-text-muted">
          Category:
          <a href={resolve('/categories/[id]', { id: product.category.id })} class="pr-link-inline">
            {product.category.name}
          </a>
        </p>
      </header>

      <section class="mb-6" aria-labelledby="reviews-heading">
        <h2 id="reviews-heading" class="pr-heading-section">
          Reviews
        </h2>
        {#if reviews.length === 0}
          <p class="pr-text-muted">No reviews yet.</p>
        {:else}
          <ul class="space-y-3">
            {#each reviews as review (review.id)}
              <li class="rounded-lg border border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
                <p class="font-medium pr-text-body">
                  Rating: {review.rating}/5
                </p>
                {#if review.text}
                  <p class="mt-1 pr-text-body text-gray-700 dark:text-gray-200">{review.text}</p>
                {/if}
                <p class="mt-1 text-sm pr-text-subtle">
                  By {review.user.name} · {formatDate(review.updated_at)}
                </p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="mb-6" aria-labelledby="purchase-history-heading">
        <h2 id="purchase-history-heading" class="pr-heading-section">
          Purchase history
        </h2>
        {#if purchases.length === 0}
          <p class="pr-text-muted">No purchases recorded.</p>
        {:else}
          <ul class="space-y-2">
            {#each purchases as purchase (purchase.id)}
              <li class="flex flex-wrap gap-x-4 gap-y-1 pr-text-body text-gray-700 dark:text-gray-200">
                <span>{formatDate(purchase.purchased_at)}</span>
                <span>{purchase.location.name}</span>
                <span>{purchase.price} €</span>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="border-t border-gray-200 pt-4" aria-label="Actions">
        <p class="pr-text-muted">
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolve() + query string; rule only accepts direct resolve() -->
          <a href={`${resolve('/manage/reviews/add')}?product_id=${product.id}`} class="pr-link-inline">
            Add review
          </a>
          <span class="mx-2">·</span>
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolve() + query string; rule only accepts direct resolve() -->
          <a href={`${resolve('/manage/purchases/add')}?product_id=${product.id}`} class="pr-link-inline">
            Add purchase
          </a>
        </p>
      </section>
    </article>
  {/if}
</main>
