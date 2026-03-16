<script lang="ts">
  /** Price amount string (e.g. "2.99"). Renders nothing when falsy. */
  let { amount = null }: { amount?: string | null } = $props();

  const displayAmount = $derived(
    amount != null && String(amount).trim() !== '' ? String(amount).trim() : null
  );
</script>

{#if displayAmount != null}
  <span
    class="text-sm pr-text-muted"
    aria-label="Price: {displayAmount} euros"
  >
    {displayAmount} €
  </span>
{/if}
