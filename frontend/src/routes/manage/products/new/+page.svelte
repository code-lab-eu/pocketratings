<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { createProduct } from '$lib/api';
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
    submitting = true;
    try {
      await createProduct({ name: n, brand: brand.trim(), category_id: categoryId });
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
