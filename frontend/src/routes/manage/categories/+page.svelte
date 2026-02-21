<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteCategory } from '$lib/api';
	import EmptyState from '$lib/EmptyState.svelte';
	import type { Category } from '$lib/types';

	let { data } = $props();
	let categories = $derived(data.categories);
	let error = $derived(data.error);
	let deletingId = $state<string | null>(null);

	async function handleDelete(c: Category) {
		if (deletingId) return;
		if (!confirm(`Delete category "${c.name}"?`)) return;
		deletingId = c.id;
		try {
			await deleteCategory(c.id);
			await goto(resolve('/manage/categories'), { invalidateAll: true });
		} catch (e) {
			alert(e instanceof Error ? e.message : String(e));
		} finally {
			deletingId = null;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a href={resolve('/manage')} class="text-gray-600 hover:text-gray-900">← Manage</a>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900">Categories</h1>
	<a
		href={resolve('/manage/categories/new')}
		class="mb-4 inline-block rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800"
	>
		New category
	</a>

	{#if error}
		<p class="text-red-600">{error}</p>
	{:else if categories.length === 0}
		<EmptyState
			message="No categories yet."
			action={{ label: 'Add your first category', href: '/manage/categories/new' }}
		/>
	{:else}
		<ul class="space-y-2">
			{#each categories as category (category.id)}
				<li class="flex min-w-0 items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3">
					<a href={resolve(`/manage/categories/${category.id}`)} class="min-h-[44px] min-w-0 flex-1 break-words py-2 text-gray-900 hover:underline">
						{category.name}
					</a>
					<button
						type="button"
						onclick={() => handleDelete(category)}
						disabled={deletingId === category.id}
						class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
						aria-label="Delete {category.name}"
					>
						{deletingId === category.id ? '…' : 'Delete'}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>
