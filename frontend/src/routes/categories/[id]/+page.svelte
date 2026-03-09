<script lang="ts">
  import { resolve } from '$app/paths';
  import { listProducts, listReviews } from '$lib/api';
  import ProductList from '$lib/ProductList.svelte';
  import SearchForm from '$lib/SearchForm.svelte';
  import type { Category, Product, Review } from '$lib/types';
  import type { ProductWithReview } from './+page';

  let { data } = $props();

  let category = $derived(data.category);
  let allChildren = $derived(category?.children ?? []);
  let notFound = $derived(data.notFound ?? false);

  let displayedItems = $state<ProductWithReview[]>([]);
  let displayedError = $state<string | null>(null);
  let searchQuery = $state('');

  $effect(() => {
    displayedItems = data.items;
    displayedError = data.error;
    searchQuery = data.query ?? '';
  });

  let childCategories = $derived(
    searchQuery.trim() === ''
      ? allChildren
      : allChildren.filter((child: Category) =>
        child.name.toLowerCase().includes(searchQuery.toLowerCase())
      )
  );

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
    if (!category) return;
    const path = resolve(`/categories/${category.id}`);
    const url = q ? `${path}?q=${encodeURIComponent(q)}` : path;
    if (typeof history !== 'undefined') {
      history.replaceState(null, '', url);
    }

    if (q === '') {
      displayedItems = data.items;
      displayedError = data.error;
      searchQuery = '';
      return;
    }

    searchQuery = q;
    try {
      const [products, reviews] = await Promise.all([
        listProducts({ category_id: category.id, q }),
        listReviews()
      ]);
      displayedItems = mergeProductsWithReviews(products, reviews);
      displayedError = null;
    } catch (e) {
      displayedError = e instanceof Error ? e.message : String(e);
    }
  }
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
  {:else if displayedError}
    <p class="text-red-600 dark:text-red-300">{displayedError}</p>
  {:else if category}
    <SearchForm
      actionUrl={resolve(`/categories/${category.id}`)}
      query={searchQuery}
      onQueryChange={onQueryChange}
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
    {:else if searchQuery.trim() !== ''}
      <p class="pr-text-muted mb-6">No categories match.</p>
    {/if}
    {#if displayedItems.length === 0}
      <p class="pr-text-muted">No products in this category.</p>
    {:else}
      <ProductList items={displayedItems} />
    {/if}
  {:else}
    <p class="pr-text-muted">Category not found.</p>
  {/if}
</main>
