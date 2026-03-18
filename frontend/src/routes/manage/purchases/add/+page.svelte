<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { createPurchase } from '$lib/api';
  import type { PurchaseFormData } from '$lib/PurchaseForm.svelte';
  import BackLink from '$lib/BackLink.svelte';
  import FormError from '$lib/FormError.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import PurchaseForm from '$lib/PurchaseForm.svelte';

  let { data } = $props();
  let products = $derived(data.products);
  let locations = $derived(data.locations);
  let prefillProductId = $derived(data.productId ?? '');
  let loadError = $derived(data.error);

  async function handleSubmit(formData: PurchaseFormData) {
    await createPurchase(formData);
    await goto(resolve('/manage/purchases'), { invalidateAll: true });
  }
</script>

<svelte:head>
  <title>Add purchase — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/purchases')} label="Purchases" />
  <PageHeading>Record purchase</PageHeading>

  {#if loadError}
    <FormError message={loadError} />
  {:else}
    <PurchaseForm
      {products}
      {locations}
      onSubmit={handleSubmit}
      submitLabel="Record"
      cancelHref={resolve('/manage/purchases')}
      initialProductId={prefillProductId}
    />
  {/if}
</main>
