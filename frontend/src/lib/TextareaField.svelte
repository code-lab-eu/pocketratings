<script lang="ts">
  interface Props {
    id: string;
    label: string;
    value: string;
    required?: boolean;
    placeholder?: string;
    rows?: number;
  }

  let {
    id,
    label,
    value = $bindable(),
    required = false,
    placeholder = '',
    rows = 3
  }: Props = $props();
</script>

<div>
  <label for={id} class="mb-1 block pr-text-label">{label}</label>
  <textarea
    {id}
    bind:value
    {required}
    placeholder={placeholder || undefined}
    {rows}
    class="pr-input"
  ></textarea>
</div>
