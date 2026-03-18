<script lang="ts">
  import { UNIT_OPTIONS } from '$lib/api';
  import InputField from '$lib/InputField.svelte';
  import Button from '$lib/Button.svelte';

  interface Props {
    idPrefix: string;
    labelValue: string;
    unit: string;
    quantity: string;
    onSubmit: (e: Event) => void;
    onCancel: () => void;
    submitting: boolean;
    submitLabel: string;
    submittingLabel: string;
  }

  let {
    idPrefix,
    labelValue = $bindable(),
    unit = $bindable(),
    quantity = $bindable(),
    onSubmit,
    onCancel,
    submitting,
    submitLabel,
    submittingLabel
  }: Props = $props();
</script>

<form onsubmit={onSubmit} class="space-y-2">
  <InputField id="{idPrefix}-label" label="Label" bind:value={labelValue} />
  <label for="{idPrefix}-unit" class="mb-1 block text-sm font-medium">Unit</label>
  <select
    id="{idPrefix}-unit"
    bind:value={unit}
    class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
  >
    {#each UNIT_OPTIONS as opt (opt.value)}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
  {#if unit !== 'none'}
    <InputField
      id="{idPrefix}-quantity"
      label="Quantity"
      type="number"
      min="0"
      bind:value={quantity}
    />
  {/if}
  <div class="flex gap-2 pt-1">
    <Button type="submit" disabled={submitting} variant="primary">
      {submitting ? submittingLabel : submitLabel}
    </Button>
    <button type="button" class="pr-btn-secondary" onclick={onCancel}>
      Cancel
    </button>
  </div>
</form>
