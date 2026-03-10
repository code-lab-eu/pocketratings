<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { updateReview } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import Button from '$lib/Button.svelte';
  import FormError from '$lib/FormError.svelte';
  import PageHeading from '$lib/PageHeading.svelte';

  let { data } = $props();
  let review = $derived(data.review);
  let loadError = $derived(data.error);

  let rating = $state(4);
  let text = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (review) {
      rating = review.rating;
      text = review.text ?? '';
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!review) return;
    error = null;
    const r = Number(rating);
    if (r < 1 || r > 5) {
      error = 'Rating must be between 1 and 5.';
      return;
    }
    submitting = true;
    try {
      await updateReview(review.id, {
        rating: r,
        text: text.trim() || null
      });
      await goto(resolve(`/products/${review.product.id}`), { invalidateAll: true });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/reviews')} label="Reviews" />

  {#if loadError}
    <FormError message={loadError} />
  {:else if review}
    <PageHeading>
      Edit review
      {#snippet description()}
        Product: {review.product.name}{review.product.brand ? ` — ${review.product.brand}` : ''}
      {/snippet}
    </PageHeading>
    <form onsubmit={handleSubmit} class="mt-4 space-y-4">
      <FormError message={error} />
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
