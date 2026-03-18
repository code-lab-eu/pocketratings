<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { updatePurchase } from '$lib/api';
  import type { PurchaseFormData } from '$lib/PurchaseForm.svelte';
  import BackLink from '$lib/BackLink.svelte';
  import FormError from '$lib/FormError.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import PurchaseForm from '$lib/PurchaseForm.svelte';

  let { data } = $props();
  let purchase = $derived(data.purchase);
  let products = $derived(data.products);
  let locations = $derived(data.locations);
  let loadError = $derived(data.error);

  let initialValues = $derived.by(() => {
    if (!purchase) return undefined;
    return {
      productId: purchase.product.id,
      variationId: purchase.variation.id,
      locationId: purchase.location.id,
      quantity: purchase.quantity,
      price: purchase.price,
      purchasedAt: new Date(purchase.purchased_at * 1000).toISOString().slice(0, 16)
    };
  });

  async function handleSubmit(formData: PurchaseFormData) {
    if (!purchase) return;
    await updatePurchase(purchase.id, formData);
    await goto(resolve('/manage/purchases'), { invalidateAll: true });
  }
</script>

<svelte:head>
  <title>Edit purchase — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/purchases')} label="Purchases" />

  {#if loadError}
    <FormError message={loadError} />
  {:else if purchase}
    <PageHeading>Edit purchase</PageHeading>
    <PurchaseForm
      {products}
      {locations}
      onSubmit={handleSubmit}
      submitLabel="Save"
      cancelHref={resolve('/manage/purchases')}
      {initialValues}
    />
  {/if}
</main>
