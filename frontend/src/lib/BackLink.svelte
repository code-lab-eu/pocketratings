<script lang="ts">
  interface Props {
    href: string;
    label: string;
  }

  let { href, label }: Props = $props();
</script>

<nav class="mb-4">
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolved at callsites -->
  <a href={href} class="pr-link-muted">← {label}</a>
</nav>
