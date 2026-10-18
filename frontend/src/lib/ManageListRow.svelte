<script lang="ts">
  /* eslint-disable svelte/no-navigation-without-resolve -- viewHref is pre-resolved by parent */
  import type { Snippet } from 'svelte';
  import { Trash2 } from 'lucide-svelte';
  import EditLink from '$lib/EditLink.svelte';
  import IconButton from '$lib/IconButton.svelte';

  interface Props {
    /** Primary text for the row (e.g. entity name or "Name — Brand"). */
    label: string;
    /** If set, the label is rendered as a link to this URL; otherwise plain text. */
    viewHref?: string;
    /** URL for the Edit icon link. */
    editHref: string;
    /** Text used in the Delete button aria-label (e.g. entity name). */
    deleteLabel: string;
    /** Called when the user confirms delete. */
    onDelete: () => void | Promise<void>;
    /** When true, the Delete button is disabled and can show a loading state. */
    deleting?: boolean;
    /** Optional extra content after the label (e.g. rating, location/date/price). */
    children?: Snippet;
    /** Optional indent depth (e.g. for category tree; applied as margin-left in rem). */
    depth?: number;
  }

  let {
    label,
    viewHref,
    editHref,
    deleteLabel,
    onDelete,
    deleting = false,
    children,
    depth = 0
  }: Props = $props();
</script>

<li
  class="flex min-w-0 items-center justify-between gap-2 pr-card"
  style={depth > 0 ? `margin-left: ${depth}rem` : undefined}
>
  <div class="min-h-[44px] min-w-0 flex-1 break-words py-2">
    {#if viewHref}
      <a href={viewHref} class="font-medium pr-text-body hover:underline">
        {label}
      </a>
    {:else}
      <span class="font-medium pr-text-body">{label}</span>
    {/if}
    {#if children}
      {@render children()}
    {/if}
  </div>
  <div class="flex shrink-0 items-center gap-1">
    <EditLink href={editHref} {label} />
    <IconButton
      tone="danger"
      onclick={onDelete}
      disabled={deleting}
      label="Delete {deleteLabel}"
    >
      {#if deleting}
        <span class="text-sm">…</span>
      {:else}
        <Trash2 size={20} />
      {/if}
    </IconButton>
  </div>
</li>
