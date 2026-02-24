<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteCategory } from '$lib/api';
	import { flattenCategories } from '$lib/categories';
	import CategoryLinkList from '$lib/CategoryLinkList.svelte';
	import EmptyState from '$lib/EmptyState.svelte';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';
	import type { Category } from '$lib/types';

	let { data } = $props();
	let categories = $derived(data.categories);
	let flat = $derived(flattenCategories(categories));
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
		<a
			href={resolve('/manage')}
			class="pr-link-muted"
			>← Manage</a
		>
	</nav>
	<PageHeading>Categories</PageHeading>
	<Button
		variant="primary"
		href={resolve('/manage/categories/new')}
		class="mb-4 inline-block"
	>
		New category
	</Button>

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if flat.length === 0}
		<EmptyState
			message="No categories yet."
			action={{ label: 'Add your first category', href: '/manage/categories/new' }}
		/>
	{:else}
		{#snippet deleteButton(category: Category)}
			<button
				type="button"
				onclick={() => handleDelete(category)}
				disabled={deletingId === category.id}
				class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50 dark:text-red-300 dark:hover:text-red-200"
				aria-label="Delete {category.name}"
			>
				{deletingId === category.id ? '…' : 'Delete'}
			</button>
		{/snippet}
		<CategoryLinkList
			items={flat}
			basePath="manage/categories"
			action={deleteButton}
		/>
	{/if}
</main>
