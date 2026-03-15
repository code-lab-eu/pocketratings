<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { createProduct, UNIT_OPTIONS } from '$lib/api';
  import { flattenCategories } from '$lib/categories';
  import BackLink from '$lib/BackLink.svelte';
  import CategorySelect from '$lib/CategorySelect.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let categories = $derived(data.categories);
  let categoryOptions = $derived(flattenCategories(categories));
  let loadError = $derived(data.error);
  let initialCategoryId = $derived(data.categoryId ?? null);

  let name = $state('');
  let brand = $state('');
  let categoryId = $state('');
  let variationLabel = $state('');
  let variationUnit = $state('none');
  let variationQuantity = $state('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  // Prefill category when opened with ?category_id= (e.g. from category edit page)
  $effect(() => {
    const id = initialCategoryId;
    const options = categoryOptions;
    if (id && options.some((o) => o.category.id === id)) {
      categoryId = id;
    }
  });

  // Clear quantity when unit is "No unit" so we don't send it
  $effect(() => {
    if (variationUnit === 'none') {
      variationQuantity = '';
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = null;
    const n = name.trim();
    if (!n) {
      error = 'Name is required.';
      return;
    }
    if (!categoryId) {
      error = 'Category is required.';
      return;
    }
    const body: Parameters<typeof createProduct>[0] = {
      name: n,
      brand: brand.trim(),
      category_id: categoryId
    };
    const qRaw = String(variationQuantity ?? '').trim();
    const q = qRaw === '' ? undefined : Number.parseInt(qRaw, 10);
    const hasQuantity = q !== undefined && !Number.isNaN(q);
    if (variationUnit !== 'none' || String(variationLabel ?? '').trim() || hasQuantity) {
      body.first_variation = {
        label: String(variationLabel ?? '').trim() || undefined,
        unit: variationUnit,
        quantity: hasQuantity ? q : undefined
      };
    }
    submitting = true;
    try {
      await createProduct(body);
      await goto(resolve('/manage/products'), { invalidateAll: true });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/products')} label="Products" />
  <PageHeading>New product</PageHeading>

  {#if loadError}
    <FormError message={loadError} />
  {:else}
    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={error} />
      <InputField id="name" label="Name" bind:value={name} required />
      <InputField id="brand" label="Brand" bind:value={brand} />
      <CategorySelect
        options={categoryOptions}
        bind:value={categoryId}
        id="category"
        label="Category"
        placeholder="Select category"
        required
      />
      <fieldset class="space-y-2">
        <legend class="text-sm font-medium">First variation (optional)</legend>
        <p class="text-sm pr-text-muted">Set size or unit for this product (e.g. 500 g, 1 L).</p>
        <InputField
          id="variation-label"
          label="Label"
          bind:value={variationLabel}
          placeholder="e.g. 500 g"
        />
        <div>
          <label for="variation-unit" class="mb-1 block text-sm font-medium">Unit</label>
          <select
            id="variation-unit"
            bind:value={variationUnit}
            class="pr-input w-full"
          >
            {#each UNIT_OPTIONS as opt (opt.value)}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
        {#if variationUnit !== 'none'}
          <InputField
            id="variation-quantity"
            label="Quantity (optional)"
            type="number"
            min="1"
            bind:value={variationQuantity}
            placeholder="e.g. 500"
          />
        {/if}
      </fieldset>
      <div class="flex gap-2">
        <Button type="submit" disabled={submitting} variant="primary">
          {submitting ? 'Creating…' : 'Create'}
        </Button>
        <Button variant="secondary" href={resolve('/manage/products')}>
          Cancel
        </Button>
      </div>
    </form>
  {/if}
</main>
