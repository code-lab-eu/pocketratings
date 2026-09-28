<script lang="ts">
  import { Pen } from 'lucide-svelte';

  interface Props {
    /** URL of the edit page (already resolved by the caller). */
    href: string;
    /** Name of the item to edit; the accessible name is "Edit {label}". */
    label: string;
  }

  let { href, label }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolved by the caller -->
<a {href} class="flex min-h-[44px] min-w-[44px] items-center justify-center pr-link-muted" aria-label="Edit {label}">
  <Pen size={20} />
</a>
