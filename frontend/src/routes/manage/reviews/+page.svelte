<script lang="ts">
	import { resolve } from '$app/paths';
	import { goto } from '$app/navigation';
	import { deleteReview } from '$lib/api';
	import PageHeading from '$lib/PageHeading.svelte';
	import Button from '$lib/Button.svelte';
	import type { Review } from '$lib/types';

	let { data } = $props();
	let reviews = $derived(data.reviews);
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
		<a
			href={resolve('/manage')}
			class="pr-link-muted"
			>← Manage</a
		>
	</nav>
	<PageHeading>Reviews</PageHeading>
	<Button
		variant="primary"
		href={resolve('/manage/reviews/add')}
		class="mb-4 inline-block"
	>
		Add review
	</Button>

	{#if error}
		<p class="text-red-600 dark:text-red-300">{error}</p>
	{:else if reviews.length === 0}
		<p class="pr-text-muted">No reviews yet.</p>
	{:else}
		<ul class="space-y-2">
			{#each reviews as review (review.id)}
				<li class="flex items-center justify-between gap-2 rounded-lg border border-gray-200 bg-white px-4 py-3 dark:border-gray-700 dark:bg-gray-800">
					<div class="min-w-0 flex-1">
						<a
							href={resolve(`/products/${review.product.id}`)}
							class="font-medium pr-text-body"
						>
							{review.product.name}
						</a>
						<span class="pr-text-muted"> — {review.rating}/5</span>
						<span class="pr-text-muted"> · {review.user.name}</span>
						{#if review.text}
							<p class="mt-1 truncate text-sm pr-text-muted">{review.text}</p>
						{/if}
					</div>
					<button
						type="button"
						onclick={() => handleDelete(review)}
						disabled={deletingId === review.id}
						class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50 dark:text-red-300 dark:hover:text-red-200"
						aria-label="Delete review"
					>
						{deletingId === review.id ? '…' : 'Delete'}
					</button>
				</li>
			{/each}
		</ul>
	{/if}
</main>
