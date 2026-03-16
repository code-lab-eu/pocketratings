<script lang="ts">
  import { resolve } from '$app/paths';
  import { ChevronDown, ChevronRight } from 'lucide-svelte';
  import type { Category } from '$lib/types';

  interface Props {
    items: { category: Category; depth: number }[];
    basePath: string;
    action?: import('svelte').Snippet<[Category]>;
    expandedIds?: Set<string>;
    onToggle?: (category: Category) => void;
  }

  let { items, basePath, action, expandedIds, onToggle }: Props = $props();

  function hasChildren(category: Category): boolean {
    return (category.children?.length ?? 0) > 0;
  }

  function isExpanded(id: string): boolean {
    return expandedIds?.has(id) ?? false;
  }
</script>

<ul class="space-y-2">
  {#each items as { category, depth } (category.id)}
    {#if action}
      <li
        class="flex min-w-0 items-center justify-between gap-2 pr-card"
        style="margin-left: {depth * 1}rem"
      >
        <div class="flex min-w-0 items-center gap-1">
          {#if onToggle && hasChildren(category)}
            <button
              type="button"
              class="inline-flex shrink-0 items-center justify-center rounded p-0.5 hover:bg-black/5 dark:hover:bg-white/10"
              aria-label={isExpanded(category.id) ? `Collapse ${category.name}` : `Expand ${category.name}`}
              onclick={() => onToggle?.(category)}
            >
              {#if isExpanded(category.id)}
                <ChevronDown size={16} aria-hidden="true" />
              {:else}
                <ChevronRight size={16} aria-hidden="true" />
              {/if}
            </button>
          {/if}
          {#if basePath === 'manage/categories'}
            <a href={resolve('/manage/categories/[id]', { id: category.id })} class="pr-list-item-link">
              {category.name}
            </a>
          {:else}
            <a href={resolve('/categories/[id]', { id: category.id })} class="pr-list-item-link">
              {category.name}
            </a>
          {/if}
        </div>
        {@render action(category)}
      </li>
    {:else}
      <li
        class="flex min-w-0 items-center gap-1"
        style="margin-left: {depth * 1}rem"
      >
        {#if onToggle && hasChildren(category)}
          <button
            type="button"
            class="inline-flex shrink-0 items-center justify-center rounded p-0.5 hover:bg-black/5 dark:hover:bg-white/10"
            aria-label={isExpanded(category.id) ? `Collapse ${category.name}` : `Expand ${category.name}`}
            onclick={() => onToggle?.(category)}
          >
            {#if isExpanded(category.id)}
              <ChevronDown size={16} aria-hidden="true" />
            {:else}
              <ChevronRight size={16} aria-hidden="true" />
            {/if}
          </button>
        {/if}
        {#if basePath === 'manage/categories'}
          <a href={resolve('/manage/categories/[id]', { id: category.id })} class="pr-card block flex-1">
            {category.name}
          </a>
        {:else}
          <a href={resolve('/categories/[id]', { id: category.id })} class="pr-card block flex-1">
            {category.name}
          </a>
        {/if}
      </li>
    {/if}
  {/each}
</ul>
