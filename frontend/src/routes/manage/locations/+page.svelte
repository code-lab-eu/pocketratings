<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteLocation } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import EmptyState from '$lib/EmptyState.svelte';
  import FormError from '$lib/FormError.svelte';
  import ManageListRow from '$lib/ManageListRow.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';
  import type { Location } from '$lib/types';

  let { data } = $props();
  let locations = $derived(data.locations);
  let error = $derived(data.error);
  let deletingId = $state<string | null>(null);

  async function handleDelete(loc: Location) {
    if (deletingId) return;
    if (!confirm(`Delete location "${loc.name}"?`)) return;
    deletingId = loc.id;
    try {
      await deleteLocation(loc.id);
      await goto(resolve('/manage/locations'), { invalidateAll: true });
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      deletingId = null;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage')} label="Manage" />
  <PageHeading>Locations</PageHeading>
  <Button
    variant="primary"
    href={resolve('/manage/locations/new')}
    class="mb-4 inline-block"
  >
    New location
  </Button>

  {#if error}
    <FormError message={error} />
  {:else if locations.length === 0}
    <EmptyState
      message="No locations yet."
      action={{ label: 'Add your first location', href: '/manage/locations/new' }}
    />
  {:else}
    <ul class="space-y-2">
      {#each locations as location (location.id)}
        <ManageListRow
          label={location.name}
          editHref={resolve('/manage/locations/[id]', { id: location.id })}
          deleteLabel={location.name}
          onDelete={() => handleDelete(location)}
          deleting={deletingId === location.id}
        />
      {/each}
    </ul>
  {/if}
</main>
