<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createReview } from '$lib/api';

	let { data } = $props();
	let products = $derived(data.products);
	let prefillProductId = $derived(data.productId ?? '');
	let loadError = $derived(data.error);

	let productId = $state('');
	let rating = $state(4);
	let text = $state('');
	let submitting = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (prefillProductId && products.some((p) => p.id === prefillProductId)) {
			productId = prefillProductId;
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = null;
		if (!productId) {
			error = 'Product is required';
			return;
		}
		const r = Number(rating);
		if (r < 1 || r > 5) {
			error = 'Rating must be between 1 and 5';
			return;
		}
		submitting = true;
		try {
			await createReview({
				product_id: productId,
				rating: r,
				text: text.trim() || undefined
			});
			await goto(resolve('/manage/reviews'), { invalidateAll: true });
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
			href={resolve('/manage/reviews')}
			class="text-gray-600 hover:text-gray-900 dark:text-gray-200 dark:hover:text-gray-50"
			>← Reviews</a
		>
	</nav>
	<h1 class="mb-4 text-2xl font-semibold text-gray-900 dark:text-gray-50">Add review</h1>

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
					for="rating"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Rating (1–5)</label
				>
				<input
					id="rating"
					type="number"
					bind:value={rating}
					min="1"
					max="5"
					step="0.5"
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
				/>
			</div>
			<div>
				<label
					for="text"
					class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-200"
					>Review (optional)</label
				>
				<textarea
					id="text"
					bind:value={text}
					rows="3"
					class="w-full rounded-lg border border-gray-300 px-3 py-2 text-gray-900 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50"
					placeholder="Your review…"
				></textarea>
			</div>
			<div class="flex gap-2">
				<button
					type="submit"
					disabled={submitting}
					class="rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800 disabled:opacity-50 dark:border-gray-600 dark:bg-gray-50 dark:text-gray-900 dark:hover:bg-gray-200"
				>
					{submitting ? 'Saving…' : 'Save'}
				</button>
				<a
					href={resolve('/manage/reviews')}
					class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-50 dark:hover:bg-gray-700"
					>
					Cancel
				</a>
			</div>
		</form>
	{/if}
</main>
