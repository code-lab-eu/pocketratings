<script lang="ts">
  import { resolve } from '$app/paths';
  import { listProducts } from '$lib/api';
  import { errorMessage } from '$lib/utils/formatters';
  import BackLink from '$lib/BackLink.svelte';
  import Breadcrumb from '$lib/Breadcrumb.svelte';
  import CategoryLinkList from '$lib/CategoryLinkList.svelte';
  import { flattenCategories, toggleExpanded } from '$lib/categories';
  import EmptyState from '$lib/EmptyState.svelte';
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

  let expandedIds = $state<string[]>([]);
  let displayedItems = $state<ProductListItem[]>([]);
  let displayedError = $state<string | null>(null);
  let searchQuery = $state('');

  $effect(() => {
    displayedItems = data.items;
    displayedError = data.error;
    searchQuery = data.query ?? '';
  });

  let isSearching = $derived(searchQuery.trim() !== '');
  let expandedSet = $derived(new Set(expandedIds));

  let childCategoriesTree = $derived(isSearching ? [] : allChildren);
  let childCategoriesFlat = $derived(
    isSearching
      ? flattenCategories(allChildren).filter(({ category: c }) =>
        c.name.toLowerCase().includes(searchQuery.toLowerCase())
      )
      : []
  );

  function hasChildrenOverride(cat: Category): boolean {
    return (cat.children?.length ?? 0) > 0;
  }

  function handleToggle(cat: Category) {
    expandedIds = toggleExpanded(expandedIds, cat.id);
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
      const products = await listProducts({ category_id: category.id, q });
      displayedItems = products.map((product) => ({ product }));
      displayedError = null;
    } catch (e) {
      displayedError = errorMessage(e);
    }
  }
</script>

<svelte:head>
  <title>{category ? `${category.name} — Pocket Ratings` : 'Category — Pocket Ratings'}</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  {#if category}
    {@const breadcrumbSegments = [
      { label: 'Home', href: resolve('/') },
      ...(category.ancestors ?? []).toReversed().map((a) => ({
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
    {#if isSearching}
      {#if childCategoriesFlat.length > 0}
        <div class="mb-6">
          <CategoryLinkList
            items={childCategoriesFlat}
            hrefFor={(id) => resolve('/categories/[id]', { id })}
          />
        </div>
      {:else}
        <EmptyState
          class="mb-6"
          icon="search"
          message="Nothing to see here. Try some other text?"
        />
      {/if}
    {:else if childCategoriesTree.length > 0}
      <div class="mb-6">
        <CategoryLinkList
          tree={childCategoriesTree}
          hrefFor={(id) => resolve('/categories/[id]', { id })}
          expandedIds={expandedSet}
          onToggle={handleToggle}
          hasChildrenOverride={hasChildrenOverride}
        />
      </div>
    {/if}
    {#if displayedItems.length === 0}
      <EmptyState
        icon="package"
        message="Empty corner. Fill it when ready."
      />
    {:else}
      <ProductList items={displayedItems} />
    {/if}
  {:else}
    <p class="pr-text-muted">Category not found.</p>
  {/if}
</main>
