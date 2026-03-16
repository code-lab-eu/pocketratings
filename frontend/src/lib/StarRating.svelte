<script lang="ts">
  import { formatRating } from '$lib/utils/formatters';

  /** Score 1-5 (display only). Renders nothing when null, undefined, or out of range. */
  let { score = null }: { score?: number | null } = $props();

  const validScore = $derived(
    typeof score === 'number' && score >= 1 && score <= 5 ? score : null
  );

  const uid = `sr-${Math.random().toString(36).slice(2, 11)}`;

  /** Fill amount 0-1 for star at index i (quarter granularity for display). */
  function fillForStar(i: number): number {
    const s = validScore;
    if (s == null) return 0;
    const fill = Math.min(1, Math.max(0, s - i));
    return Math.round(fill * 4) / 4;
  }
</script>

{#if validScore != null}
  <span
    class="inline-flex items-center gap-0.5 text-[1rem] pr-rating"
    aria-label="Rating: {formatRating(validScore)} out of 5"
  >
    <span aria-hidden="true" class="inline-flex items-center gap-0.5">
      {#each [0, 1, 2, 3, 4] as i (i)}
        {@const fill = fillForStar(i)}
        <span class="inline-block w-4 h-4 flex-shrink-0" role="img" aria-hidden="true">
          {#if fill >= 1}
            <svg
              class="w-full h-full text-inherit"
              viewBox="0 0 24 24"
              fill="currentColor"
              aria-hidden="true"
            >
              <path
                d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
              />
            </svg>
          {:else if fill > 0}
            <svg
              class="w-full h-full"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <defs>
                <linearGradient id="star-fill-{uid}-{i}" x1="0" x2="1" y1="0" y2="0">
                  <stop offset="0" stop-color="var(--pr-primary-500)" />
                  <stop offset={fill} stop-color="var(--pr-primary-500)" />
                  <stop offset={fill} stop-color="var(--pr-text-subtle)" />
                  <stop offset="1" stop-color="var(--pr-text-subtle)" />
                </linearGradient>
              </defs>
              <path
                d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
                fill="url(#star-fill-{uid}-{i})"
              />
            </svg>
          {:else}
            <svg
              class="w-full h-full text-[var(--pr-text-subtle)]"
              viewBox="0 0 24 24"
              fill="currentColor"
              aria-hidden="true"
            >
              <path
                d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"
              />
            </svg>
          {/if}
        </span>
      {/each}
    </span>
  </span>
{/if}
