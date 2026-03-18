<script lang="ts">
  import { getProductVariations } from '$lib/api';
  import type { Location, Product, ProductVariation } from '$lib/types';
  import { errorMessage, formatProductDisplayName, formatVariationDisplay } from '$lib/utils/formatters';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import Select from '$lib/Select.svelte';
  import Button from '$lib/Button.svelte';

  export interface PurchaseFormData {
    product_id: string;
    variation_id?: string;
    location_id: string;
    quantity: number;
    price: string;
    purchased_at?: string;
  }

  interface Props {
    products: Product[];
    locations: Location[];
    onSubmit: (data: PurchaseFormData) => Promise<void>;
    submitLabel: string;
    cancelHref: string;
    /** Pre-select product (add page with query param). */
    initialProductId?: string;
    /** Pre-fill all fields from existing purchase (edit mode). */
    initialValues?: {
      productId: string;
      variationId: string;
      locationId: string;
      quantity: number;
      price: string;
      purchasedAt: string;
    };
  }

  let {
    products,
    locations,
    onSubmit,
    submitLabel,
    cancelHref,
    initialProductId = '',
    initialValues
  }: Props = $props();

  let productId = $state('');
  let variationId = $state('');
  let locationId = $state('');
  let quantity = $state(1);
  let price = $state('');
  let purchasedAt = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);
  let variations = $state<ProductVariation[]>([]);
  let variationsLoading = $state(false);

  $effect(() => {
    if (initialValues) {
      productId = initialValues.productId;
      variationId = initialValues.variationId;
      locationId = initialValues.locationId;
      quantity = initialValues.quantity;
      price = initialValues.price;
      purchasedAt = initialValues.purchasedAt;
    } else {
      if (initialProductId && products.some((p) => p.id === initialProductId)) {
        productId = initialProductId;
      }
      if (!purchasedAt) {
        purchasedAt = new Date().toISOString().slice(0, 16);
      }
    }
  });

  $effect(() => {
    if (!productId) {
      variations = [];
      if (!initialValues) variationId = '';
      return;
    }
    let cancelled = false;
    variationsLoading = true;
    getProductVariations(productId)
      .then((list) => {
        if (cancelled) return;
        variations = list;
        if (initialValues) {
          const ids = new Set(list.map((v) => v.id));
          if (list.length && !ids.has(variationId)) variationId = list[0].id;
        } else {
          variationId = list.length > 0 ? list[0].id : '';
        }
      })
      .catch(() => {
        if (!cancelled) variations = [];
      })
      .finally(() => {
        if (!cancelled) variationsLoading = false;
      });
    return () => {
      cancelled = true;
    };
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = null;
    if (!productId || !locationId) {
      error = 'Product and location are required.';
      return;
    }
    const q = Math.floor(quantity);
    if (q < 1) {
      error = 'Quantity must be at least 1.';
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
      await onSubmit({
        product_id: productId,
        variation_id: variationId || undefined,
        location_id: locationId,
        quantity: q,
        price: priceVal,
        purchased_at: at
      });
    } catch (e) {
      error = errorMessage(e);
    } finally {
      submitting = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="space-y-4">
  <FormError message={error} />
  <Select
    id="product"
    label="Product"
    options={products.map((p) => ({
      value: p.id,
      label: formatProductDisplayName(p)
    }))}
    bind:value={productId}
    placeholder="Select product"
    required
  />
  {#if productId}
    <Select
      id="variation"
      label="Variation"
      options={variations.map((v) => ({ value: v.id, label: formatVariationDisplay(v) }))}
      bind:value={variationId}
      placeholder={variationsLoading ? 'Loading...' : 'Select variation'}
      disabled={variationsLoading || variations.length === 0}
    />
  {/if}
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
      {submitting ? 'Saving...' : submitLabel}
    </Button>
    <Button variant="secondary" href={cancelHref}>
      Cancel
    </Button>
  </div>
</form>
