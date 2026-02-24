<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deletePurchase } from '$lib/api';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';
	import type { Purchase } from '$lib/types';

	let { data } = $props();
	let purchases = $derived(data.purchases);
	let productMap = $derived(data.productMap);
	let locationMap = $derived(data.locationMap);
	let error = $derived(data.error);
	let deletingId = $state<string | null>(null);

	function formatDate(ts: number): string {
		return new Date(ts * 1000).toLocaleDateString();
	}

	async function handleDelete(p: Purchase) {
		if (deletingId) return;
		if (!confirm('Delete this purchase?')) return;
		deletingId = p.id;
		try {
			await deletePurchase(p.id);
			await goto(resolve('/manage/purchases'), { invalidateAll: true });
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
	<PageHeading>Purchases</PageHeading>
	<Button
		variant="primary"
		href={resolve('/manage/purchases/add')}
		class="mb-4 inline-block"
	>
		Record purchase
	</Button>

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if purchases.length === 0}
		<p class="text-gray-600 dark:text-gray-200">No purchases yet.</p>
	{:else}
		<ul class="space-y-2">
			{#each purchases as purchase (purchase.id)}
				<li class="flex items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
					<div class="min-w-0 flex-1">
						<span class="font-medium text-gray-900 dark:text-gray-50">{productMap.get(purchase.product_id)?.name ?? purchase.product_id}</span>
						<span class="text-gray-600 dark:text-gray-200">
							— {locationMap.get(purchase.location_id)?.name ?? purchase.location_id} · {formatDate(purchase.purchased_at)} · {purchase.price}€
						</span>
					</div>
					<button
						type="button"
						onclick={() => handleDelete(purchase)}
						disabled={deletingId === purchase.id}
						class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50 dark:text-red-300 dark:hover:text-red-200"
						aria-label="Delete purchase"
					>
						{deletingId === purchase.id ? '…' : 'Delete'}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>
