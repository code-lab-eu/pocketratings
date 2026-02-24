<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createPurchase } from '$lib/api';

	let { data } = $props();
	let products = $derived(data.products);
	let locations = $derived(data.locations);
	let prefillProductId = $derived(data.productId ?? '');
	let loadError = $derived(data.error);

	let productId = $state('');
	let locationId = $state('');
	let quantity = $state(1);
	let price = $state('');
	let purchasedAt = $state('');
	let submitting = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (prefillProductId && products.some((p) => p.id === prefillProductId)) {
			productId = prefillProductId;
		}
		if (!purchasedAt) {
			const now = new Date();
			purchasedAt = now.toISOString().slice(0, 16);
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;
		if (!productId || !locationId) {
			error = 'Product and location are required';
			return;
		}
		const q = Math.floor(quantity);
		if (q < 1) {
			error = 'Quantity must be at least 1';
			return;
		}
		const priceVal = price.trim();
		if (!priceVal) {
			error = 'Price is required';
			return;
		}
		submitting = true;
		try {
			const at = purchasedAt ? new Date(purchasedAt).toISOString() : undefined;
			await createPurchase({
				product_id: productId,
				location_id: locationId,
				quantity: q,
				price: priceVal,
				purchased_at: at
			});
			await goto(resolve('/manage/purchases'), { invalidateAll: true });
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			submitting = false;
		}
	}
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
	<nav class="mb-4">
		<a
			href={resolve('/manage/purchases')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Purchases</a
		>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50">Record purchase</h1>

	{#if loadError}
		<p class="text-red-600 dark:text-red-300">{loadError}</p>
	{:else}
		<form onsubmit={handleSubmit} class="space-y-4">
			{#if error}
				<p class="text-red-600 dark:text-red-300">{error}</p>
			{/if}
			<div>
				<label
					for="product"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Product</label
				>
				<select
					id="product"
					bind:value={productId}
					required
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
				>
					<option value="">Select product</option>
					{#each products as p (p.id)}
						<option value={p.id}>{p.name}{p.brand ? ` — ${p.brand}` : ''}</option>
					{/each}
				</select>
			</div>
			<div>
				<label
					for="location"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Location</label
				>
				<select
					id="location"
					bind:value={locationId}
					required
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
				>
					<option value="">Select location</option>
					{#each locations as loc (loc.id)}
						<option value={loc.id}>{loc.name}</option>
					{/each}
				</select>
			</div>
			<div>
				<label
					for="quantity"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Quantity</label
				>
				<input
					id="quantity"
					type="number"
					bind:value={quantity}
					min="1"
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
				/>
			</div>
			<div>
				<label
					for="price"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Price (EUR)</label
				>
				<input
					id="price"
					type="text"
					bind:value={price}
					placeholder="2.99"
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
					inputmode="decimal"
				/>
			</div>
			<div>
				<label
					for="purchased_at"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Date</label
				>
				<input
					id="purchased_at"
					type="datetime-local"
					bind:value={purchasedAt}
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
				/>
			</div>
			<div class="flex gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50 dark:border-gray-600 dark:bg-gray-50 dark:text-gray-900 dark:hover:bg-gray-200"
				>
					{submitting ? 'Saving…' : 'Record'}
				</button>
				<a
					href={resolve('/manage/purchases')}
					class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50 dark:hover:bg-gray-700"
					>
					Cancel
				</a>
			</div>
		</form>
	{/if}
</main>
