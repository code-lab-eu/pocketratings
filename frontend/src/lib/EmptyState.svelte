<script lang="ts">
  import {
    Folder,
    MapPin,
    Package,
    Search,
    ShoppingCart,
    Star
  } from 'lucide-svelte';

  export type EmptyStateIcon =
    | 'search'
    | 'package'
    | 'star'
    | 'folder'
    | 'cart'
    | 'pin';

  interface Props {
    /** Short message when the list or section is empty. */
    message: string;
    /** Optional link text and final URL (caller runs resolve() when needed). */
    action?: { label: string; href: string };
    /** Lucide icon above the message (decorative; message carries meaning for assistive tech). */
    icon?: EmptyStateIcon;
    /** Extra classes on the root (e.g. layout spacing). */
    class?: string;
  }

  let { message, action, icon, class: className = '' }: Props = $props();

  const iconSize = 40;
  const iconStroke = 1.5;

  function iconAccent(i: EmptyStateIcon): string {
    // Keep icons monochrome SVGs, but make the icon language feel more lively.
    // Lucide uses `currentColor`, so we only need to set a per-icon color.
    switch (i) {
      case 'search':
        return 'var(--pr-info)';
      case 'package':
        return 'var(--pr-warning)';
      case 'star':
        return 'var(--pr-primary-500)';
      case 'folder':
        return 'var(--pr-success)';
      case 'cart':
        return 'var(--pr-error)';
      case 'pin':
        return 'var(--pr-primary-600)';
    }

    return 'var(--pr-text-muted)';
  }
</script>

<div class="pr-empty-state pr-empty-state-enter {className}">
  {#if icon}
    {@const accent = iconAccent(icon)}
    <div class="mb-3 flex justify-center sm:justify-start" aria-hidden="true">
      <div
        class="relative flex h-10 w-10 items-center justify-center rounded-full"
        style={`color: ${accent};`}
      >
        <div
          class="absolute inset-0 rounded-full"
          style={`background-color: ${accent}; opacity: 0.12;`}
          aria-hidden="true"
        ></div>

        {#if icon === 'search'}
          <Search
            class="relative"
            size={iconSize}
            strokeWidth={iconStroke}
          />
        {:else if icon === 'package'}
          <Package class="relative" size={iconSize} strokeWidth={iconStroke} />
        {:else if icon === 'star'}
          <Star class="relative" size={iconSize} strokeWidth={iconStroke} />
        {:else if icon === 'folder'}
          <Folder class="relative" size={iconSize} strokeWidth={iconStroke} />
        {:else if icon === 'cart'}
          <ShoppingCart class="relative" size={iconSize} strokeWidth={iconStroke} />
        {:else if icon === 'pin'}
          <MapPin class="relative" size={iconSize} strokeWidth={iconStroke} />
        {/if}
      </div>
    </div>
  {/if}
  <p class="pr-text-muted">{message}</p>
  {#if action}
    <p class="mt-2">
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href pre-resolved by caller -->
      <a href={action.href} class="pr-link-inline">{action.label}</a>
    </p>
  {/if}
</div>
