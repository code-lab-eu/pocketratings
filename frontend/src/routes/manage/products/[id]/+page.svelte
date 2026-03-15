<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import {
    deleteProduct,
    updateProduct,
    getProductVariations,
    createVariation,
    updateVariation,
    deleteVariation,
    UNIT_OPTIONS
  } from '$lib/api';
  import type { ProductVariation } from '$lib/types';
  import { formatVariationDisplay } from '$lib/utils/formatters';
  import { flattenCategories } from '$lib/categories';
  import BackLink from '$lib/BackLink.svelte';
  import CategorySelect from '$lib/CategorySelect.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let product = $derived(data.product);
  let categories = $derived(data.categories);
  let categoryOptions = $derived(flattenCategories(categories));
  let error = $derived(data.error);
  let notFound = $derived(data.notFound ?? false);

  let name = $state('');
  let brand = $state('');
  let categoryId = $state('');
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  let variations = $state<ProductVariation[]>([]);
  let editingId = $state<string | null>(null);
  let addVariationOpen = $state(false);
  let addLabel = $state('');
  let addUnit = $state('none');
  let addQuantity = $state('');
  let addSubmitting = $state(false);
  let editLabel = $state('');
  let editUnit = $state('none');
  let editQuantity = $state('');
  let editSubmitting = $state(false);

  $effect(() => {
    if (product) {
      name = product.name;
      brand = product.brand;
      categoryId = product.category.id;
    }
  });

  $effect(() => {
    if (data.variations && Array.isArray(data.variations)) {
      variations = data.variations;
    }
  });

  async function refetchVariations() {
    if (!product) return;
    variations = await getProductVariations(product.id);
  }

  function canDeleteVariation(v: ProductVariation): boolean {
    const hasPurchases = (v.purchase_count ?? 0) > 0;
    const isLast = variations.length <= 1;
    return !hasPurchases && !isLast;
  }

  function deleteVariationTooltip(v: ProductVariation): string {
    if ((v.purchase_count ?? 0) > 0) {
      return 'Cannot delete: this variation has purchases.';
    }
    if (variations.length <= 1) {
      return 'Cannot delete: product must have at least one variation.';
    }
    return 'Delete variation';
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!product) return;
    formError = null;
    const n = name.trim();
    if (!n) {
      formError = 'Name is required.';
      return;
    }
    if (!categoryId) {
      formError = 'Category is required.';
      return;
    }
    submitting = true;
    try {
      await updateProduct(product.id, { name: n, brand: brand.trim(), category_id: categoryId });
      await goto(resolve('/manage/products'), { invalidateAll: true });
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function handleDelete() {
    if (!product) return;
    if (!confirm(`Delete product "${product.name}"?`)) return;
    try {
      await deleteProduct(product.id);
      await goto(resolve('/manage/products'), { invalidateAll: true });
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    }
  }

  function openAddVariation() {
    addVariationOpen = true;
    addLabel = '';
    addUnit = 'none';
    addQuantity = '';
    formError = null;
  }

  async function submitAddVariation(e: Event) {
    e.preventDefault();
    if (!product) return;
    formError = null;
    const unit = addUnit;
    const qRaw = String(addQuantity ?? '').trim();
    const quantity =
      unit === 'none' ? undefined : (qRaw === '' ? undefined : Number(qRaw));
    if (unit !== 'none' && quantity !== undefined && (Number.isNaN(quantity) || quantity < 0)) {
      formError = 'Quantity must be a non-negative number.';
      return;
    }
    addSubmitting = true;
    try {
      await createVariation(product.id, {
        label: String(addLabel ?? '').trim() || undefined,
        unit,
        quantity: quantity ?? null
      });
      await refetchVariations();
      addVariationOpen = false;
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      addSubmitting = false;
    }
  }

  function startEdit(v: ProductVariation) {
    editingId = v.id;
    editLabel = v.label;
    editUnit = v.unit;
    editQuantity = v.quantity != null ? String(v.quantity) : '';
    formError = null;
  }

  function cancelEdit() {
    editingId = null;
  }

  async function submitEdit(e: Event) {
    e.preventDefault();
    if (!editingId) return;
    formError = null;
    const unit = editUnit;
    const qRaw = String(editQuantity ?? '').trim();
    const quantity =
      unit === 'none' ? undefined : (qRaw === '' ? undefined : Number(qRaw));
    if (unit !== 'none' && quantity !== undefined && (Number.isNaN(quantity) || quantity < 0)) {
      formError = 'Quantity must be a non-negative number.';
      return;
    }
    editSubmitting = true;
    try {
      await updateVariation(editingId, {
        label: String(editLabel ?? '').trim() || undefined,
        unit,
        quantity: quantity ?? null
      });
      await refetchVariations();
      editingId = null;
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      editSubmitting = false;
    }
  }

  async function handleDeleteVariation(v: ProductVariation) {
    if (!canDeleteVariation(v)) return;
    if (!confirm(`Delete variation "${formatVariationDisplay(v)}"?`)) return;
    formError = null;
    try {
      await deleteVariation(v.id);
      await refetchVariations();
    } catch (e) {
      formError = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/products')} label="Products" />

  {#if notFound}
    <p class="pr-text-muted">Product not found.</p>
    <p class="mt-2">
      <a
        href={resolve('/manage/products')}
        class="pr-link-inline"
        >Back to products</a
      >
    </p>
  {:else if error}
    <FormError message={error} />
  {:else if product}
    <PageHeading>Edit product</PageHeading>

    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={formError} />
      <InputField id="name" label="Name" bind:value={name} required />
      <InputField id="brand" label="Brand" bind:value={brand} />
      <CategorySelect
        options={categoryOptions}
        bind:value={categoryId}
        id="category"
        label="Category"
        placeholder=""
        required
      />
      <div class="flex flex-wrap gap-2">
        <Button type="submit" disabled={submitting} variant="primary">
          {submitting ? 'Saving…' : 'Save'}
        </Button>
        <Button variant="secondary" href={resolve('/manage/products')}>
          Cancel
        </Button>
        <button
          type="button"
          onclick={handleDelete}
          class="rounded-lg border border-red-300 px-4 py-2 text-red-700 hover:bg-red-50 dark:border-red-500 dark:bg-transparent dark:text-red-300 dark:hover:bg-red-950"
        >
          Delete
        </button>
      </div>
    </form>

    <section class="mt-8 border-t border-gray-200 pt-8 dark:border-gray-700">
      <h2 class="pr-heading-section mb-1">Variations</h2>
      <p class="pr-text-muted mb-4 text-sm">
        Sizes or units for this product (e.g. 500 g, 1 L). Purchases are linked to a variation.
      </p>
      <FormError message={formError} />
      <ul class="space-y-2">
        {#each variations as v (v.id)}
          {#if editingId === v.id}
            <li class="rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-600 dark:bg-gray-800/50">
              <form onsubmit={submitEdit} class="space-y-2">
                <InputField id="edit-label-{v.id}" label="Label" bind:value={editLabel} />
                <label for="edit-unit-{v.id}" class="mb-1 block text-sm font-medium">Unit</label>
                <select
                  id="edit-unit-{v.id}"
                  bind:value={editUnit}
                  class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
                >
                  {#each UNIT_OPTIONS as opt (opt.value)}
                    <option value={opt.value}>{opt.label}</option>
                  {/each}
                </select>
                {#if editUnit !== 'none'}
                  <InputField
                    id="edit-quantity-{v.id}"
                    label="Quantity"
                    type="number"
                    min="0"
                    bind:value={editQuantity}
                  />
                {/if}
                <div class="flex gap-2 pt-1">
                  <Button type="submit" disabled={editSubmitting} variant="primary">
                    {editSubmitting ? 'Saving…' : 'Save'}
                  </Button>
                  <button
                    type="button"
                    class="pr-btn-secondary"
                    onclick={cancelEdit}
                  >
                    Cancel
                  </button>
                </div>
              </form>
            </li>
          {:else}
            <li
              class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-gray-200 px-3 py-2 dark:border-gray-600"
            >
              <span class="font-medium">{formatVariationDisplay(v)}</span>
              <div class="flex gap-2">
                <button
                  type="button"
                  class="pr-btn-secondary !py-1.5"
                  onclick={() => startEdit(v)}
                >
                  Edit
                </button>
                <button
                  type="button"
                  title={deleteVariationTooltip(v)}
                  disabled={!canDeleteVariation(v)}
                  onclick={() => handleDeleteVariation(v)}
                  class="rounded border border-red-300 px-3 py-1.5 text-sm text-red-700 disabled:cursor-not-allowed disabled:opacity-50 hover:bg-red-50 dark:border-red-500 dark:text-red-300 dark:hover:bg-red-950"
                >
                  Delete
                </button>
              </div>
            </li>
          {/if}
        {/each}
      </ul>
      {#if addVariationOpen}
        <div class="mt-4 rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-600 dark:bg-gray-800/50">
          <form onsubmit={submitAddVariation} class="space-y-2">
            <InputField id="add-label" label="Label" bind:value={addLabel} />
            <label for="add-unit" class="mb-1 block text-sm font-medium">Unit</label>
            <select
              id="add-unit"
              bind:value={addUnit}
              class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
            >
              {#each UNIT_OPTIONS as opt (opt.value)}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
            {#if addUnit !== 'none'}
              <InputField
                id="add-quantity"
                label="Quantity"
                type="number"
                min="0"
                bind:value={addQuantity}
              />
            {/if}
            <div class="flex gap-2 pt-1">
              <Button type="submit" disabled={addSubmitting} variant="primary">
                {addSubmitting ? 'Adding…' : 'Add variation'}
              </Button>
              <button
                type="button"
                class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-100 dark:border-gray-600 dark:hover:bg-gray-700"
                onclick={() => (addVariationOpen = false)}
              >
                Cancel
              </button>
            </div>
          </form>
        </div>
      {:else}
        <button
          type="button"
          class="pr-btn-secondary mt-4"
          onclick={openAddVariation}
        >
          Add variation
        </button>
      {/if}
    </section>
  {:else}
    <p class="pr-text-muted">Product not found.</p>
  {/if}
</main>
