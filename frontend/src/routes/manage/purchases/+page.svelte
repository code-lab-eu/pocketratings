<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deletePurchase } from '$lib/api';
  import ManageListRow from '$lib/ManageListRow.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';
  import type { Purchase } from '$lib/types';

  let { data } = $props();
  let purchases = $derived(data.purchases);
  let error = $derived(data.error);
  let deletingId = $state<string | null>(null);

  function purchaseLabel(p: Purchase): string {
    const { product } = p;
    return product.brand ? `${product.name} — ${product.brand}` : product.name;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString();
  }

  async function handleDelete(p: Purchase) {
    if (deletingId) return;
    if (!confirm('Delete this purchase?')) return;
    deletingId = p.id;
    try {
      await deletePurchase(p.id);
      await goto(resolve('/manage/purchases'), { invalidateAll: true });
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      deletingId = null;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <nav class="mb-4">
    <a
      href={resolve('/manage')}
      class="pr-link-muted"
      >← Manage</a
    >
  </nav>
  <PageHeading>Purchases</PageHeading>
  <Button
    variant="primary"
    href={resolve('/manage/purchases/add')}
    class="mb-4 inline-block"
  >
    Record purchase
  </Button>

  {#if error}
    <p class="text-red-600 dark:text-red-300">{error}</p>
  {:else if purchases.length === 0}
    <p class="pr-text-muted">No purchases yet.</p>
  {:else}
    <ul class="space-y-2">
      {#each purchases as purchase (purchase.id)}
        <ManageListRow
          label={purchaseLabel(purchase)}
          editHref={resolve('/manage/purchases/[id]', { id: purchase.id })}
          deleteLabel="purchase"
          onDelete={() => handleDelete(purchase)}
          deleting={deletingId === purchase.id}
        >
          <span class="pr-text-muted">
            — {purchase.location.name} · {formatDate(purchase.purchased_at)} · {purchase.price}€
          </span>
        </ManageListRow>
      {/each}
    </ul>
  {/if}
</main>
