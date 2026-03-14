<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    id: string;
    label: string;
    options: Option[];
    value: string;
    placeholder?: string;
    required?: boolean;
    disabled?: boolean;
  }

  let {
    id,
    label,
    options,
    value = $bindable(),
    placeholder = '',
    required = false,
    disabled = false
  }: Props = $props();
</script>

<div>
  <label for={id} class="mb-1 block pr-text-label">{label}</label>
  <select {id} bind:value {required} {disabled} class="pr-input" autocomplete="off">
    {#if placeholder}
      <option value="">{placeholder}</option>
    {/if}
    {#each options as opt (opt.value)}
      <option value={opt.value}>{opt.label}</option>
    {/each}
  </select>
</div>
