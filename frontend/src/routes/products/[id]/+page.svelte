<script lang="ts">
  import { resolve } from '$app/paths';
  import BackLink from '$lib/BackLink.svelte';
  import Breadcrumb from '$lib/Breadcrumb.svelte';
  import EmptyState from '$lib/EmptyState.svelte';
  import FormError from '$lib/FormError.svelte';
  import { formatDate, formatProductDisplayName, formatRating, formatVariationDisplay } from '$lib/utils/formatters';
  import NotFoundMessage from '$lib/NotFoundMessage.svelte';

  let { data } = $props();

  let product = $derived(data.product);
  let reviews = $derived(data.reviews);
  let purchases = $derived(data.purchases);
  let error = $derived(data.error);
  let notFound = $derived(data.notFound ?? false);

  /** Groups purchases by variation (only variations with at least one purchase).
   *  Most recent purchases first; variation order = first occurrence in that order. */
  let purchasesByVariation = $derived.by(() => {
    const list = [...(purchases ?? [])].sort(
      (a, b) => (b.purchased_at ?? 0) - (a.purchased_at ?? 0)
    );
    if (list.length === 0) return [];
    const groups: {
      variationId: string;
      displayName: string;
      purchases: (typeof list)[number][];
    }[] = [];
    for (const p of list) {
      const vid = p.variation?.id ?? '';
      let g = groups.find((x) => x.variationId === vid);
      if (!g) {
        g = {
          variationId: vid,
          displayName: formatVariationDisplay(p.variation ?? { label: '', unit: 'none' }),
          purchases: []
        };
        groups.push(g);
      }
      g.purchases.push(p);
    }
    return groups;
  });

</script>

<svelte:head>
  <title>
    {product ? `${formatProductDisplayName(product)} — Pocket Ratings` : 'Product — Pocket Ratings'}
  </title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  {#if product}
    {@const breadcrumbSegments = [
      { label: 'Home', href: resolve('/') },
      ...(product.category.ancestors ?? []).toReversed().map((a) => ({
        label: a.name,
        href: resolve(`/categories/${a.id}`)
      })),
      { label: product.category.name, href: resolve('/categories/[id]', { id: product.category.id }) },
      { label: formatProductDisplayName(product) }
    ]}
    <Breadcrumb segments={breadcrumbSegments} />
  {:else}
    <BackLink href={resolve('/')} label="Home" />
  {/if}

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
        <h1 class="pr-heading-page break-words">
          {product.name}
        </h1>
        {#if product.brand}
          <p class="pr-product-brand">{product.brand}</p>
        {/if}
      </header>

      <section class="mb-6" aria-labelledby="reviews-heading">
        <h2 id="reviews-heading" class="pr-heading-section">
          Reviews
        </h2>
        {#if reviews.length === 0}
          <EmptyState
            icon="star"
            message="No reviews yet. Add one when you are ready."
          />
        {:else}
          <ul class="space-y-3">
            {#each reviews as review (review.id)}
              <li class="pr-panel">
                <p class="font-medium pr-text-body pr-rating">
                  Rating: {formatRating(review.rating)}/5
                </p>
                {#if review.text}
                  <p class="mt-1 pr-text-body">{review.text}</p>
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
        {#if (purchases ?? []).length === 0}
          <EmptyState
            icon="cart"
            message="This pocket is empty, why don't you go and buy one?"
          />
        {:else}
          {#each purchasesByVariation as group (group.variationId)}
            {#if purchasesByVariation.length > 1}
              <h3 class="mt-4 mb-2 text-sm font-medium pr-text-body first:mt-0">
                {group.displayName}
              </h3>
            {/if}
            <ul class="space-y-2" class:mt-0={purchasesByVariation.length === 1}>
              {#each group.purchases as purchase (purchase.id)}
                <li class="flex flex-wrap gap-x-4 gap-y-1 pr-text-body">
                  <span>{formatDate(purchase.purchased_at)}</span>
                  <span>{purchase.location.name}</span>
                  <span class="pr-text-muted" title="Quantity">×{purchase.quantity}</span>
                  <span>{purchase.price} €</span>
                </li>
              {/each}
            </ul>
          {/each}
        {/if}
      </section>

      <section class="pr-divider pt-4" aria-label="Actions">
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
