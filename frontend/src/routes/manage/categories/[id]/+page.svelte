<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteCategory, updateCategory } from '$lib/api';
  import { errorMessage } from '$lib/utils/formatters';
  import { flattenCategories } from '$lib/categories';
  import BackLink from '$lib/BackLink.svelte';
  import NotFoundMessage from '$lib/NotFoundMessage.svelte';
  import CategorySelect from '$lib/CategorySelect.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let category = $derived(data.category);
  let categories = $derived(data.categories);
  let parentOptions = $derived(
    category ? flattenCategories(categories).filter(({ category: c }) => c.id !== category.id) : []
  );
  let error = $derived(data.error);
  let notFound = $derived(data.notFound ?? false);

  let name = $state('');
  let parentId = $state('');
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  // $effect() runs a side effect when any reactive dependency (e.g. category) changes.
  // Here we sync the form fields (name, parentId) whenever the loaded category changes
  // (e.g. after navigation to this page or after load re-runs).
  $effect(() => {
    if (category) {
      name = category.name;
      // Direct parent is first in ancestors (closest first)
      parentId = category.ancestors?.length ? category.ancestors[0].id : '';
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!category) return;
    formError = null;
    const n = name.trim();
    if (!n) {
      formError = 'Name is required.';
      return;
    }
    submitting = true;
    try {
      await updateCategory(category.id, { name: n, parent_id: parentId || null });
      await goto(resolve('/manage/categories'), { invalidateAll: true });
    } catch (e) {
      formError = errorMessage(e);
    } finally {
      submitting = false;
    }
  }

  async function handleDelete() {
    if (!category) return;
    if (!confirm(`Delete category "${category.name}"?`)) return;
    try {
      await deleteCategory(category.id);
      await goto(resolve('/manage/categories'), { invalidateAll: true });
    } catch (e) {
      formError = errorMessage(e);
    }
  }
</script>

<svelte:head>
  <title>
    {category ? `Edit category: ${category.name} — Pocket Ratings` : 'Category — Pocket Ratings'}
  </title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/categories')} label="Categories" />

  {#if notFound}
    <NotFoundMessage
      message="Category not found."
      backHref={resolve('/manage/categories')}
      backLabel="Back to categories"
    />
  {:else if error}
    <FormError message={error} />
  {:else if category}
    <PageHeading>Edit category</PageHeading>

    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={formError} />
      <InputField id="name" label="Name" bind:value={name} required />
      <CategorySelect
        options={parentOptions}
        bind:value={parentId}
        id="parent"
        label="Parent (optional)"
        placeholder="None"
      />
      <div class="flex flex-wrap gap-2">
        <Button type="submit" disabled={submitting} variant="primary">
          {submitting ? 'Saving…' : 'Save'}
        </Button>
        <Button variant="secondary" href={resolve('/manage/categories')}>
          Cancel
        </Button>
        <button type="button" onclick={handleDelete} class="pr-btn-danger">
          Delete
        </button>
      </div>
    </form>
  {/if}
</main>
