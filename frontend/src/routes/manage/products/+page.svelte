<script lang="ts">
  import { resolve } from '$app/paths';
  import { goto } from '$app/navigation';
  import { deleteProduct } from '$lib/api';
  import BackLink from '$lib/BackLink.svelte';
  import EmptyState from '$lib/EmptyState.svelte';
  import FormError from '$lib/FormError.svelte';
  import ManageListRow from '$lib/ManageListRow.svelte';
  import PageHeading from '$lib/PageHeading.svelte';
  import Button from '$lib/Button.svelte';
  import type { Product } from '$lib/types';

  let { data } = $props();
  let products = $derived(data.products);
  let error = $derived(data.error);
  let deletingId = $state<string | null>(null);

  function productLabel(p: Product): string {
    return p.brand ? `${p.name} — ${p.brand}` : p.name;
  }

  async function handleDelete(p: Product) {
    if (deletingId) return;
    if (!confirm(`Delete product "${p.name}"?`)) return;
    deletingId = p.id;
    try {
      await deleteProduct(p.id);
      await goto(resolve('/manage/products'), { invalidateAll: true });
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      deletingId = null;
    }
  }
</script>

<main class="mx-auto max-w-2xl px-4 py-8">
  <BackLink href={resolve('/manage')} label="Manage" />
  <PageHeading>Products</PageHeading>
  <Button
    variant="primary"
    href={resolve('/manage/products/new')}
    class="mb-4 inline-block"
  >
    New product
  </Button>

  {#if error}
    <FormError message={error} />
  {:else if products.length === 0}
    <EmptyState
      message="No products yet."
      action={{ label: 'Add your first product', href: '/manage/products/new' }}
    />
  {:else}
    <ul class="space-y-2">
      {#each products as product (product.id)}
        <ManageListRow
          label={productLabel(product)}
          viewHref={resolve('/products/[id]', { id: product.id })}
          editHref={resolve('/manage/products/[id]', { id: product.id })}
          deleteLabel={product.name}
          onDelete={() => handleDelete(product)}
          deleting={deletingId === product.id}
        />
      {/each}
    </ul>
  {/if}
</main>
