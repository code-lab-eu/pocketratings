<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { updatePurchase } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Select from '$lib/Select.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let purchase = $derived(data.purchase);
  let products = $derived(data.products);
  let locations = $derived(data.locations);
  let loadError = $derived(data.error);

  let productId = $state('');
  let locationId = $state('');
  let quantity = $state(1);
  let price = $state('');
  let purchasedAt = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (purchase) {
      productId = purchase.product.id;
      locationId = purchase.location.id;
      quantity = purchase.quantity;
      price = purchase.price;
      purchasedAt = new Date(purchase.purchased_at * 1000).toISOString().slice(0, 16);
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!purchase) return;
    error = null;
    if (!productId || !locationId) {
      error = 'Product and location are required.';
      return;
    }
    const q = Math.floor(quantity);
    if (q < 1) {
      error = 'Quantity must be at least 1';
      return;
    }
    const priceVal = price.trim();
    if (!priceVal) {
      error = 'Price is required.';
      return;
    }
    submitting = true;
    try {
      const at = purchasedAt ? new Date(purchasedAt).toISOString() : undefined;
      await updatePurchase(purchase.id, {
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
  <BackLink href={resolve('/manage/purchases')} label="Purchases" />

  {#if loadError}
    <FormError message={loadError} />
  {:else if purchase}
    <PageHeading>Edit purchase</PageHeading>
    <form onsubmit={handleSubmit} class="mt-4 space-y-4">
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
      <Select
        id="location"
        label="Location"
        options={locations.map((loc) => ({ value: loc.id, label: loc.name }))}
        bind:value={locationId}
        placeholder="Select location"
        required
      />
      <InputField
        id="quantity"
        label="Quantity"
        type="number"
        bind:value={quantity}
        min={1}
      />
      <InputField
        id="price"
        label="Price (EUR)"
        bind:value={price}
        placeholder="2.99"
        inputmode="decimal"
      />
      <InputField
        id="purchased_at"
        label="Date"
        type="datetime-local"
        bind:value={purchasedAt}
      />
      <div class="flex gap-2">
        <Button type="submit" disabled={submitting} variant="primary">
          {submitting ? 'Saving…' : 'Save'}
        </Button>
        <Button variant="secondary" href={resolve('/manage/purchases')}>
          Cancel
        </Button>
      </div>
    </form>
  {/if}
</main>
