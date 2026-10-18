<!--
  Password input with a show/hide toggle. It repeats InputField's label and input rather than
  wrapping it, because the toggle has to be positioned over the input itself, not over the
  label-plus-input block that InputField renders.
-->
<script lang="ts">
  import { Eye, EyeOff } from 'lucide-svelte';
  import IconButton from '$lib/IconButton.svelte';

  interface Props {
    id: string;
    label: string;
    value: string;
    required?: boolean;
    autocomplete?: 'current-password' | 'new-password' | string;
  }

  let {
    id,
    label,
    value = $bindable(),
    required = false,
    autocomplete
  }: Props = $props();

  let visible = $state(false);

  const autocompleteValue = $derived(
    (autocomplete ?? 'off') as HTMLInputElement['autocomplete']
  );
</script>

<div>
  <label for={id} class="mb-1 block pr-text-label">{label}</label>
  <div class="relative">
    <input
      {id}
      bind:value
      type={visible ? 'text' : 'password'}
      {required}
      autocomplete={autocompleteValue}
      class="pr-input pe-12"
    />
    <IconButton
      onclick={() => (visible = !visible)}
      pressed={visible}
      label={visible ? 'Hide password' : 'Show password'}
      class="absolute inset-y-0 right-0 rounded-r-lg"
    >
      {#if visible}
        <EyeOff size={20} aria-hidden="true" />
      {:else}
        <Eye size={20} aria-hidden="true" />
      {/if}
    </IconButton>
  </div>
</div>
