<script lang="ts">
  import { ArrowLeft } from 'lucide-svelte';

  interface Props {
    href: string;
    label: string;
  }

  let { href, label }: Props = $props();
</script>

<nav class="mb-4">
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolved at callsites -->
  <a href={href} class="pr-link-muted inline-flex items-center gap-1.5">
    <ArrowLeft size={16} aria-hidden="true" />
    {label}
  </a>
</nav>
