<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteLocation, updateLocation } from '$lib/api';
  import { errorMessage } from '$lib/utils/formatters';
  import BackLink from '$lib/BackLink.svelte';
  import NotFoundMessage from '$lib/NotFoundMessage.svelte';
  import FormError from '$lib/FormError.svelte';
  import InputField from '$lib/InputField.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';

  let { data } = $props();
  let location = $derived(data.location);
  let error = $derived(data.error);
  let notFound = $derived(data.notFound ?? false);

  let name = $state('');
  let submitting = $state(false);
  let formError = $state<string | null>(null);

  $effect(() => {
    if (location) {
      name = location.name;
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!location) return;
    formError = null;
    const n = name.trim();
    if (!n) {
      formError = 'Name is required.';
      return;
    }
    submitting = true;
    try {
      await updateLocation(location.id, { name: n });
      await goto(resolve('/manage/locations'), { invalidateAll: true });
    } catch (e) {
      formError = errorMessage(e);
    } finally {
      submitting = false;
    }
  }

  async function handleDelete() {
    if (!location) return;
    if (!confirm(`Delete location "${location.name}"?`)) return;
    try {
      await deleteLocation(location.id);
      await goto(resolve('/manage/locations'), { invalidateAll: true });
    } catch (e) {
      formError = errorMessage(e);
    }
  }
</script>

<svelte:head>
  <title>
    {location
      ? `Edit location: ${location.name} — Pocket Ratings`
      : 'Location — Pocket Ratings'}
  </title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage/locations')} label="Locations" />

  {#if notFound}
    <NotFoundMessage
      message="Location not found."
      backHref={resolve('/manage/locations')}
      backLabel="Back to locations"
    />
  {:else if error}
    <FormError message={error} />
  {:else if location}
    <PageHeading>Edit location</PageHeading>

    <form onsubmit={handleSubmit} class="space-y-4">
      <FormError message={formError} />
      <InputField id="name" label="Name" bind:value={name} required />
      <div class="flex flex-wrap gap-2">
        <Button type="submit" disabled={submitting} variant="primary">
          {submitting ? 'Saving…' : 'Save'}
        </Button>
        <Button variant="secondary" href={resolve('/manage/locations')}>
          Cancel
        </Button>
        <button type="button" onclick={handleDelete} class="pr-btn-danger">
          Delete
        </button>
      </div>
    </form>
  {/if}
</main>
