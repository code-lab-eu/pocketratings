<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { createReview } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Select from '$lib/Select.svelte';
  import TextareaField from '$lib/TextareaField.svelte';
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
      error = 'Product is required.';
      return;
    }
    const r = Number(rating);
    if (r < 1 || r > 5) {
      error = 'Rating must be between 1 and 5.';
      return;
    }
    const ratingRounded = Math.round(r * 10) / 10;
    submitting = true;
    try {
      await createReview({
        product_id: productId,
        rating: ratingRounded,
        text: text.trim() || undefined
      });
      await goto(resolve(`/products/${productId}`), { invalidateAll: true });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/reviews')} label="Reviews" />
  <PageHeading>Add review</PageHeading>

  {#if loadError}
    <FormError message={loadError} />
  {:else}
    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={error} />
      <Select
        id="product"
        label="Product"
        options={products.map((p) => ({
          value: p.id,
          label: p.brand ? `${p.name} — ${p.brand}` : p.name
        }))}
        bind:value={productId}
        placeholder="Select product"
        required
      />
      <InputField
        id="rating"
        label="Rating (1–5)"
        type="number"
        bind:value={rating}
        min={1}
        max={5}
        step={0.1}
      />
      <TextareaField
        id="text"
        label="Review (optional)"
        bind:value={text}
        rows={3}
        placeholder="Your review…"
      />
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
