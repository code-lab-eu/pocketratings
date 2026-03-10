<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteReview } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import EmptyState from '$lib/EmptyState.svelte';
  import FormError from '$lib/FormError.svelte';
  import ManageListRow from '$lib/ManageListRow.svelte';
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
  <BackLink href={resolve('/manage')} label="Manage" />
  <PageHeading>Reviews</PageHeading>
  <Button
    variant="primary"
    href={resolve('/manage/reviews/add')}
    class="mb-4 inline-block"
  >
    Add review
  </Button>

  {#if error}
    <FormError message={error} />
  {:else if reviews.length === 0}
    <EmptyState
      message="No reviews yet."
      action={{ label: 'Add your first review', href: '/manage/reviews/add' }}
    />
  {:else}
    <ul class="space-y-2">
      {#each reviews as review (review.id)}
        <ManageListRow
          label={review.product.name}
          viewHref={resolve('/products/[id]', { id: review.product.id })}
          editHref={resolve('/manage/reviews/[id]', { id: review.id })}
          deleteLabel="review"
          onDelete={() => handleDelete(review)}
          deleting={deletingId === review.id}
        >
          <span class="pr-text-muted"> — {review.rating}/5</span>
          <span class="pr-text-muted"> · {review.user.name}</span>
          {#if review.text}
            <p class="mt-1 truncate text-sm pr-text-muted">{review.text}</p>
          {/if}
        </ManageListRow>
      {/each}
    </ul>
  {/if}
</main>
