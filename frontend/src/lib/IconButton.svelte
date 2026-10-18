<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Accessible name (aria-label). Required: an icon has no visible text. */
    label: string;
    onclick?: (e: MouseEvent) => void;
    /** Toggle state (aria-pressed); the attribute is omitted when undefined. */
    pressed?: boolean;
    /** Disclosure state (aria-expanded); the attribute is omitted when undefined. */
    expanded?: boolean;
    title?: string;
    disabled?: boolean;
    /** `danger` is for destructive actions such as delete. */
    tone?: 'muted' | 'danger';
    /** Positioning only (e.g. absolute placement, negative margins). */
    class?: string;
    /** The icon. */
    children: Snippet;
  }

  let {
    label,
    onclick,
    pressed,
    expanded,
    title,
    disabled = false,
    tone = 'muted',
    class: className = '',
    children
  }: Props = $props();

  const toneClass = $derived(
    tone === 'danger'
      ? 'text-red-600 hover:text-red-800 dark:text-red-300 dark:hover:text-red-200'
      : 'pr-link-muted'
  );

  const classes = $derived(
    `pr-btn-icon ${toneClass} disabled:opacity-50${className ? ` ${className}` : ''}`
  );
</script>

<button
  type="button"
  class={classes}
  aria-label={label}
  aria-pressed={pressed}
  aria-expanded={expanded}
  {title}
  {disabled}
  {onclick}
>
  {@render children()}
</button>
