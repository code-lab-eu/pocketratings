<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { createCategory } from '$lib/api';
  import { flattenCategories } from '$lib/categories';
  import BackLink from '$lib/BackLink.svelte';
  import CategorySelect from '$lib/CategorySelect.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let categories = $derived(data.categories);
  let parentOptions = $derived(flattenCategories(categories));
  let loadError = $derived(data.error);

  let name = $state('');
  let parentId = $state<string>('');
  let submitting = $state(false);
  let error = $state<string | null>(null);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = null;
    const n = name.trim();
    if (!n) {
      error = 'Name is required.';
      return;
    }
    submitting = true;
    try {
      await createCategory({ name: n, parent_id: parentId || null });
      await goto(resolve('/manage/categories'), { invalidateAll: true });
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>New category — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/categories')} label="Categories" />
  <PageHeading>New category</PageHeading>

  {#if loadError}
    <FormError message={loadError} />
  {:else}
  <form onsubmit={handleSubmit} class="space-y-4">
    <FormError message={error} />
    <InputField id="name" label="Name" bind:value={name} required />
    <CategorySelect
      options={parentOptions}
      bind:value={parentId}
      id="parent"
      label="Parent (optional)"
      placeholder="None"
    />
    <div class="flex gap-2">
      <Button type="submit" disabled={submitting} variant="primary">
        {submitting ? 'Creating…' : 'Create'}
      </Button>
      <Button variant="secondary" href={resolve('/manage/categories')}>
        Cancel
      </Button>
    </div>
  </form>
  {/if}
</main>
