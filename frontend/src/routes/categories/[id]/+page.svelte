<script lang="ts">
  import { resolve } from '$app/paths';
  import { listProducts } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import Breadcrumb from '$lib/Breadcrumb.svelte';
  import FormError from '$lib/FormError.svelte';
  import NotFoundMessage from '$lib/NotFoundMessage.svelte';
  import ProductList from '$lib/ProductList.svelte';
  import SearchForm from '$lib/SearchForm.svelte';
  import type { Category } from '$lib/types';
  import type { ProductListItem } from './+page';

  let { data } = $props();

  let category = $derived(data.category);
  let allChildren = $derived(category?.children ?? []);
  let notFound = $derived(data.notFound ?? false);

  let displayedItems = $state<ProductListItem[]>([]);
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
      const products = await listProducts({ category_id: category.id, q });
      displayedItems = products.map((product) => ({ product }));
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
  {#if category}
    {@const breadcrumbSegments = [
      { label: 'Home', href: resolve('/') },
      ...(category.ancestors ?? []).reverse().map((a) => ({
        label: a.name,
        href: resolve(`/categories/${a.id}`)
      })),
      { label: category.name }
    ]}
    <Breadcrumb segments={breadcrumbSegments} />
  {:else}
    <BackLink href={resolve('/')} label="Home" />
  {/if}

  {#if notFound}
    <NotFoundMessage
      message="Category not found."
      backHref={resolve('/')}
      backLabel="Back to home"
    />
  {:else if displayedError}
    <FormError message={displayedError} />
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
