<script lang="ts">
  import { resolve } from '$app/paths';
  import { listProducts } from '$lib/api';
  import CategoryLinkList from '$lib/CategoryLinkList.svelte';
  import FormError from '$lib/FormError.svelte';
  import ProductList from '$lib/ProductList.svelte';
  import SearchForm from '$lib/SearchForm.svelte';
  import type { Category } from '$lib/types';
  import type { ProductListItem } from './+page';

  type CategoryWithDepth = { category: Category; depth: number };

  let { data } = $props();

  let displayedCategories = $state<CategoryWithDepth[]>([]);
  let displayedItems = $state<ProductListItem[]>([]);
  let displayedError = $state<string | null>(null);
  let searchQuery = $state('');

  $effect(() => {
    displayedCategories = data.categories;
    displayedItems = data.items;
    displayedError = data.error;
    searchQuery = data.query;
  });

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
      const products = await listProducts({ q });
      const full = data.fullCategories ?? data.categories;
      displayedCategories = full.filter(({ category }) =>
        category.name.toLowerCase().includes(q.toLowerCase())
      );
      displayedItems = products.map((product) => ({ product }));
      displayedError = null;
    } catch (e) {
      displayedError = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<svelte:head>
  <title>Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <SearchForm
    actionUrl={resolve('/')}
    query={searchQuery}
    onQueryChange={onQueryChange}
  />

  {#if displayedError}
    <FormError message={displayedError} />
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
