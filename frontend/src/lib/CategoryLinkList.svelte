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
				class="flex min-w-0 items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3"
				style="margin-left: {depth * 1}rem"
			>
				{#if basePath === 'manage/categories'}
					<a
						href={resolve('/manage/categories/[id]', { id: category.id })}
						class="min-h-[44px] min-w-0 flex-1 break-words py-2 text-gray-900 hover:underline"
					>
						{category.name}
					</a>
				{:else}
					<a
						href={resolve('/categories/[id]', { id: category.id })}
						class="min-h-[44px] min-w-0 flex-1 break-words py-2 text-gray-900 hover:underline"
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
						class="block rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50"
					>
						{category.name}
					</a>
				{:else}
					<a
						href={resolve('/categories/[id]', { id: category.id })}
						class="block rounded-lg border border-gray-200 bg-white px-4 py-3 text-gray-900 hover:bg-gray-50"
					>
						{category.name}
					</a>
				{/if}
			</li>
		{/if}
	{/each}
</ul>
