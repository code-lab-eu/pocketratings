<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deletePurchase } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import EmptyState from '$lib/EmptyState.svelte';
  import FormError from '$lib/FormError.svelte';
  import { errorMessage, formatDate, formatProductDisplayName, formatVariationDisplay } from '$lib/utils/formatters';
  import ManageListRow from '$lib/ManageListRow.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';
  import type { Purchase } from '$lib/types';

  let { data } = $props();
  let purchases = $derived(data.purchases);
  let error = $derived(data.error);
  let deletingId = $state<string | null>(null);


  async function handleDelete(p: Purchase) {
    if (deletingId) return;
    if (!confirm('Delete this purchase?')) return;
    deletingId = p.id;
    try {
      await deletePurchase(p.id);
      await goto(resolve('/manage/purchases'), { invalidateAll: true });
    } catch (e) {
      alert(errorMessage(e));
    } finally {
      deletingId = null;
    }
  }
</script>

<svelte:head>
  <title>Purchases — Pocket Ratings</title>
</svelte:head>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage')} label="Manage" />
  <PageHeading>Purchases</PageHeading>
  <Button
    variant="primary"
    href={resolve('/manage/purchases/add')}
    class="mb-4 inline-block"
  >
    Record purchase
  </Button>

  {#if error}
    <FormError message={error} />
  {:else if purchases.length === 0}
    <EmptyState
      icon="cart"
      message="Do you have a receipt? Type it in!"
      action={{ label: 'Record your first purchase', href: '/manage/purchases/add' }}
    />
  {:else}
    <ul class="space-y-2">
      {#each purchases as purchase (purchase.id)}
        <ManageListRow
          label={formatProductDisplayName(purchase.product)}
          editHref={resolve('/manage/purchases/[id]', { id: purchase.id })}
          deleteLabel="purchase"
          onDelete={() => handleDelete(purchase)}
          deleting={deletingId === purchase.id}
        >
          <span class="pr-text-muted">
            — {formatVariationDisplay(purchase.variation)} · {purchase.location.name} ·
            {formatDate(purchase.purchased_at)} · {purchase.price}€
          </span>
        </ManageListRow>
      {/each}
    </ul>
  {/if}
</main>
