<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteReview } from '$lib/api';
	import type { Review } from '$lib/types';

	let { data } = $props();
	let reviews = $derived(data.reviews);
	let productMap = $derived(data.productMap);
	let error = $derived(data.error);
	let deletingId = $state<string | null>(null);

	async function handleDelete(r: Review) {
		if (deletingId) return;
		if (!confirm('Delete this review?')) return;
		deletingId = r.id;
		try {
			await deleteReview(r.id);
			await goto(resolve('/manage/reviews'), { invalidateAll: true });
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
	<h1 class="mb-4 text-2xl font-semibold text-gray-900">Reviews</h1>
	<a
		href={resolve('/manage/reviews/add')}
		class="mb-4 inline-block rounded-lg bg-gray-900 px-4 py-2 text-white hover:bg-gray-800"
	>
		Add review
	</a>

	{#if error}
		<p class="text-red-600">{error}</p>
	{:else if reviews.length === 0}
		<p class="text-gray-600">No reviews yet.</p>
	{:else}
		<ul class="space-y-2">
			{#each reviews as review (review.id)}
				<li class="flex items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3">
					<div class="min-w-0 flex-1">
						<a
							href={resolve(`/products/${review.product_id}`)}
							class="font-medium text-gray-900 hover:underline"
						>
							{productMap.get(review.product_id)?.name ?? review.product_id}
						</a>
						<span class="text-gray-600"> — {review.rating}/5</span>
						{#if review.text}
							<p class="mt-1 truncate text-sm text-gray-600">{review.text}</p>
						{/if}
					</div>
					<button
						type="button"
						onclick={() => handleDelete(review)}
						disabled={deletingId === review.id}
						class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
						aria-label="Delete review"
					>
						{deletingId === review.id ? '…' : 'Delete'}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>
