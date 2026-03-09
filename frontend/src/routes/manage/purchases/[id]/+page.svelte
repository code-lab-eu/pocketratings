<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { updatePurchase } from '$lib/api';
  import PageHeading from '$lib/PageHeading.svelte';
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
      error = 'Product and location are required';
      return;
    }
    const q = Math.floor(quantity);
    if (q < 1) {
      error = 'Quantity must be at least 1';
      return;
    }
    const priceVal = price.trim();
    if (!priceVal) {
      error = 'Price is required';
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
  <nav class="mb-4">
    <a
      href={resolve('/manage/purchases')}
      class="pr-link-muted"
      >← Purchases</a
    >
  </nav>

  {#if loadError}
    <p class="text-red-600 dark:text-red-300">{loadError}</p>
  {:else if purchase}
    <PageHeading>Edit purchase</PageHeading>
    <form onsubmit={handleSubmit} class="mt-4 space-y-4">
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
        <label for="location" class="mb-1 block pr-text-label">Location</label>
        <select
          id="location"
          bind:value={locationId}
          required
          class="pr-input"
        >
          <option value="">Select location</option>
          {#each locations as loc (loc.id)}
            <option value={loc.id}>{loc.name}</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="quantity" class="mb-1 block pr-text-label">Quantity</label>
        <input
          id="quantity"
          type="number"
          bind:value={quantity}
          min="1"
          class="pr-input"
        />
      </div>
      <div>
        <label for="price" class="mb-1 block pr-text-label">Price (EUR)</label>
        <input
          id="price"
          type="text"
          bind:value={price}
          placeholder="2.99"
          class="pr-input"
          inputmode="decimal"
        />
      </div>
      <div>
        <label for="purchased_at" class="mb-1 block pr-text-label">Date</label>
        <input
          id="purchased_at"
          type="datetime-local"
          bind:value={purchasedAt}
          class="pr-input"
        />
      </div>
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
