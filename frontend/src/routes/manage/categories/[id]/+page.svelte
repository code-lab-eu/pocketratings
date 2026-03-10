<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteCategory, updateCategory } from '$lib/api';
  import { flattenCategories } from '$lib/categories';
  import BackLink from '$lib/BackLink.svelte';
  import CategorySelect from '$lib/CategorySelect.svelte';
  import FormError from '$lib/FormError.svelte';
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
      formError = e instanceof Error ? e.message : String(e);
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
      formError = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/categories')} label="Categories" />

  {#if notFound}
    <p class="pr-text-muted">Category not found.</p>
    <p class="mt-2">
      <a
        href={resolve('/manage/categories')}
        class="pr-link-inline"
        >Back to categories</a
      >
    </p>
  {:else if error}
    <FormError message={error} />
  {:else if category}
    <PageHeading>Edit category</PageHeading>

    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={formError} />
      <div>
        <label for="name" class="mb-1 block pr-text-label">Name</label>
        <input
          id="name"
          type="text"
          bind:value={name}
          required
          class="pr-input"
          autocomplete="off"
        />
      </div>
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
        <button
          type="button"
          onclick={handleDelete}
          class="rounded-lg border border-red-300 px-4 py-2 text-red-700 hover:bg-red-50 dark:border-red-500 dark:bg-transparent dark:text-red-300 dark:hover:bg-red-950"
        >
          Delete
        </button>
      </div>
    </form>
  {:else}
    <p class="pr-text-muted">Category not found.</p>
  {/if}
</main>
