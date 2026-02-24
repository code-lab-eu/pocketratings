<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteProduct } from '$lib/api';
	import type { Product } from '$lib/types';

	let { data } = $props();
	let products = $derived(data.products);
	let error = $derived(data.error);
	let deletingId = $state<string | null>(null);

	async function handleDelete(p: Product) {
		if (deletingId) return;
		if (!confirm(`Delete product "${p.name}"?`)) return;
		deletingId = p.id;
		try {
			await deleteProduct(p.id);
			await goto(resolve('/manage/products'), { invalidateAll: true });
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
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Manage</a
		>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50">Products</h1>
	<a
		href={resolve('/manage/products/new')}
		class="mb-4 inline-block rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 dark:border-gray-600 dark:bg-gray-50 dark:text-gray-900 dark:hover:bg-gray-200"
	>
		New product
	</a>

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if products.length === 0}
		<p class="text-gray-600 dark:text-gray-200">No products yet.</p>
	{:else}
		<ul class="space-y-2">
			{#each products as product (product.id)}
				<li class="flex min-w-0 items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
					<a
						href={resolve(`/manage/products/${product.id}`)}
						class="min-h-[44px] min-w-0 flex-1 break-words py-2 text-gray-900 hover:underline dark:text-gray-50"
					>
						<span class="font-medium">{product.name}</span>
						{#if product.brand}
							<span class="text-gray-600 dark:text-gray-200"> — {product.brand}</span>
						{/if}
					</a>
					<button
						type="button"
						onclick={() => handleDelete(product)}
						disabled={deletingId === product.id}
						class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50 dark:text-red-300 dark:hover:text-red-200"
						aria-label="Delete {product.name}"
					>
						{deletingId === product.id ? '…' : 'Delete'}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>
