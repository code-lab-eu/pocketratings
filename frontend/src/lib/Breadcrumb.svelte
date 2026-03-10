<script lang="ts">
  interface Segment {
    label: string;
    href?: string;
  }

  interface Props {
    segments: Segment[];
  }

  let { segments }: Props = $props();
</script>

<nav class="mb-4" aria-label="Breadcrumb">
  <ol class="flex flex-wrap items-center gap-x-1 text-sm pr-text-muted">
    {#each segments as segment, i (`${segment.label}-${segment.href ?? 'current'}`)}
      <li class="flex items-center gap-x-1" aria-current={segment.href === undefined ? 'page' : undefined}>
        {#if i > 0}
          <span aria-hidden="true">/</span>
        {/if}
        {#if segment.href}
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is resolved at callsites -->
          <a href={segment.href} class="pr-link-muted">{segment.label}</a>
        {:else}
          <span>{segment.label}</span>
        {/if}
      </li>
    {/each}
  </ol>
</nav>
