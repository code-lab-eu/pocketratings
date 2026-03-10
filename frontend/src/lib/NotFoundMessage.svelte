<script lang="ts">
  interface Props {
    message: string;
    backHref: string;
    backLabel: string;
  }

  let { message, backHref, backLabel }: Props = $props();
</script>

<p class="pr-text-muted">{message}</p>
<p class="mt-2">
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- backHref is resolved at callsites -->
  <a href={backHref} class="pr-link-inline">{backLabel}</a>
</p>
