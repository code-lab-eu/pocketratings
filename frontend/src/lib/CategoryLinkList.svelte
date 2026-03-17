<script lang="ts">
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import { slide } from 'svelte/transition';
  import type { Category } from '$lib/types';

  interface Props {
    items?: { category: Category; depth: number }[];
    tree?: Category[];
    hrefFor: (id: string) => string;
    action?: import('svelte').Snippet<[Category]>;
    expandedIds?: Set<string>;
    onToggle?: (category: Category) => void;
    /** When provided, used instead of category.children?.length for chevron visibility (e.g. lazy-load). */
    hasChildrenOverride?: (category: Category) => boolean;
  }

  let { items = [], tree, hrefFor, action, expandedIds, onToggle, hasChildrenOverride }: Props = $props();

  function hasChildren(category: Category): boolean {
    const fromData = (category.children?.length ?? 0) > 0;
    if (hasChildrenOverride) return hasChildrenOverride(category) || fromData;
    return fromData;
  }

  function isExpanded(id: string): boolean {
    return expandedIds?.has(id) ?? false;
  }

  function handleChevronClick(e: MouseEvent, category: Category) {
    e.stopPropagation();
    e.preventDefault();
    onToggle?.(category);
  }
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- hrefs are provided by the caller via hrefFor prop; resolve() is called on the calling side -->
{#snippet treeCategoryLink(cat: Category, depth: number)}
  <a
    href={hrefFor(cat.id)}
    class="pr-card flex items-center gap-1"
    style="padding-left: calc(0.75rem + {depth} * 1.25rem)"
  >
    {#if hasChildren(cat)}
      <button
        type="button"
        class="inline-flex shrink-0 items-center justify-center rounded p-0.5 hover:bg-black/5 dark:hover:bg-white/10"
        aria-label={isExpanded(cat.id) ? `Collapse ${cat.name}` : `Expand ${cat.name}`}
        onclick={(e: MouseEvent) => handleChevronClick(e, cat)}
      >
        {#if isExpanded(cat.id)}
          <ChevronDown size={16} aria-hidden="true" />
        {:else}
          <ChevronRight size={16} aria-hidden="true" />
        {/if}
      </button>
    {:else}
      <span class="inline-block w-5 shrink-0" data-testid="chevron-spacer"></span>
    {/if}
    {cat.name}
  </a>
{/snippet}

{#snippet treeNodes(categories: Category[], depth: number)}
  {#each categories as category, i (category.id)}
    <li class={depth === 0 && i === 0 ? '' : 'mt-2'}>
      {@render treeCategoryLink(category, depth)}
      {#if isExpanded(category.id) && category.children?.length}
        <ul class="flex flex-col" transition:slide={{ duration: 200 }}>
          {@render treeNodes(category.children, depth + 1)}
        </ul>
      {/if}
    </li>
  {/each}
{/snippet}

{#if tree && onToggle}
  <ul class="flex flex-col">
    {@render treeNodes(tree, 0)}
  </ul>
{:else}
  <ul class="flex flex-col">
    {#each items as { category, depth } (category.id)}
      {#if action}
        <li
          class="mt-2 flex min-w-0 items-center justify-between gap-2 pr-card first:mt-0"
          style="margin-left: {depth * 1}rem"
        >
          <div class="flex min-w-0 items-center gap-1">
            <a href={hrefFor(category.id)} class="pr-list-item-link">
              {category.name}
            </a>
          </div>
          {@render action(category)}
        </li>
      {:else}
        <li class="mt-2 first:mt-0">
          <a
            href={hrefFor(category.id)}
            class="pr-card flex items-center gap-1"
            style="padding-left: calc(0.75rem + {depth} * 1.25rem)"
          >
            {category.name}
          </a>
        </li>
      {/if}
    {/each}
  </ul>
{/if}
