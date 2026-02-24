<script lang="ts">
	import { resolve } from '$app/paths';
	import type { Category } from '$lib/types';

	interface Props {
		items: { category: Category; depth: number }[];
		basePath: string;
		action?: import('svelte').Snippet<[Category]>;
	}

	let { items, basePath, action }: Props = $props();
</script>

<ul class="space-y-2">
	{#each items as { category, depth } (category.id)}
		{#if action}
			<li
				class="flex min-w-0 items-center justify-between gap-2 pr-card"
				style="margin-left: {depth * 1}rem"
			>
				{#if basePath === 'manage/categories'}
					<a
						href={resolve('/manage/categories/[id]', { id: category.id })}
						class="pr-list-item-link"
					>
						{category.name}
					</a>
				{:else}
					<a
						href={resolve('/categories/[id]', { id: category.id })}
						class="pr-list-item-link"
					>
						{category.name}
					</a>
				{/if}
				{@render action(category)}
			</li>
		{:else}
			<li style="margin-left: {depth * 1}rem">
				{#if basePath === 'manage/categories'}
					<a
						href={resolve('/manage/categories/[id]', { id: category.id })}
						class="pr-card block"
					>
						{category.name}
					</a>
				{:else}
					<a
						href={resolve('/categories/[id]', { id: category.id })}
						class="pr-card block"
					>
						{category.name}
					</a>
				{/if}
			</li>
		{/if}
	{/each}
</ul>
