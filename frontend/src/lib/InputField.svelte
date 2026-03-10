<script lang="ts">
  type InputType =
    | 'text'
    | 'number'
    | 'email'
    | 'password'
    | 'datetime-local';

  type InputMode =
    | 'search'
    | 'url'
    | 'none'
    | 'text'
    | 'tel'
    | 'email'
    | 'numeric'
    | 'decimal';

  interface Props {
    id: string;
    label: string;
    value: string | number;
    type?: InputType;
    required?: boolean;
    placeholder?: string;
    min?: number | string;
    max?: number | string;
    step?: number | string;
    autocomplete?: 'off' | 'on' | 'email' | 'current-password' | string;
    inputmode?: InputMode;
  }

  let {
    id,
    label,
    value = $bindable(),
    type = 'text',
    required = false,
    placeholder = '',
    min,
    max,
    step,
    autocomplete,
    inputmode
  }: Props = $props();

  const autocompleteValue = $derived(
    (autocomplete ?? 'off') as HTMLInputElement['autocomplete']
  );
</script>

<div>
  <label for={id} class="mb-1 block pr-text-label">{label}</label>
  <input
    {id}
    bind:value
    {type}
    {required}
    placeholder={placeholder || undefined}
    {min}
    {max}
    {step}
    autocomplete={autocompleteValue}
    inputmode={inputmode}
    class="pr-input"
  />
</div>
