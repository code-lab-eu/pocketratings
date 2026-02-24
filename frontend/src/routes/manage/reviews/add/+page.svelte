<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { createReview } from '$lib/api';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';

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
			class="pr-link-muted"
			>← Reviews</a
		>
	</nav>
	<PageHeading>Add review</PageHeading>

	{#if loadError}
		<p class="text-red-600 dark:text-red-300">{loadError}</p>
	{:else}
		<form onsubmit={handleSubmit} class="space-y-4">
			{#if error}
				<p class="text-red-600 dark:text-red-300">{error}</p>
			{/if}
			<div>
				<label for="product" class="mb-1 block pr-text-label">Product</label>
				<select
					id="product"
					bind:value={productId}
					required
					class="pr-input"
				>
					<option value="">Select product</option>
					{#each products as p (p.id)}
						<option value={p.id}>{p.name}{p.brand ? ` — ${p.brand}` : ''}</option>
					{/each}
				</select>
			</div>
			<div>
				<label for="rating" class="mb-1 block pr-text-label">Rating (1–5)</label>
				<input
					id="rating"
					type="number"
					bind:value={rating}
					min="1"
					max="5"
					step="0.5"
					class="pr-input"
				/>
			</div>
			<div>
				<label for="text" class="mb-1 block pr-text-label">Review (optional)</label>
				<textarea
					id="text"
					bind:value={text}
					rows="3"
					class="pr-input"
					placeholder="Your review…"
				></textarea>
			</div>
			<div class="flex gap-2">
				<Button type="submit" disabled={submitting} variant="primary">
					{submitting ? 'Saving…' : 'Save'}
				</Button>
				<Button variant="secondary" href={resolve('/manage/reviews')}>
					Cancel
				</Button>
			</div>
		</form>
	{/if}
</main>
