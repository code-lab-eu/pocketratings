<script lang="ts">
  import { formatRating } from '$lib/utils/formatters';

  interface Props {
    value: number;
    id: string;
    label?: string;
  }

  let { value = $bindable(3), id, label = 'Rating (1-5)' }: Props = $props();

  const STAR_PATH = 'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z';

  const uid = `sri-${Math.random().toString(36).slice(2, 11)}`;

  let pulsing = $state(false);

  function fillForStar(i: number): number {
    return Math.min(1, Math.max(0, value - i));
  }

  function triggerPulse() {
    pulsing = false;
    requestAnimationFrame(() => {
      pulsing = true;
    });
  }

  function handleAnimationEnd() {
    pulsing = false;
  }

  let thumbPercent = $derived(((value - 1) / 4) * 100);
</script>

<div>
  <label for={id} class="mb-1 block pr-text-label">{label}</label>
  <div class="relative inline-flex flex-col items-start pt-6">
    <span
      class="pr-text-label pointer-events-none absolute top-0 text-sm font-semibold"
      style:left="{thumbPercent}%"
      style:transform="translateX(-50%)"
    >
      {formatRating(value)}
    </span>
    <div
      class="relative"
      class:pr-star-pulse={pulsing}
      onanimationend={handleAnimationEnd}
    >
      <span class="inline-flex items-center gap-1" aria-hidden="true">
        {#each [0, 1, 2, 3, 4] as i (i)}
          {@const fill = fillForStar(i)}
          <span class="inline-block h-8 w-8 flex-shrink-0">
            {#if fill >= 1}
              <svg
                class="h-full w-full text-[var(--pr-primary-500)]"
                viewBox="0 0 24 24"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d={STAR_PATH} />
              </svg>
            {:else if fill > 0}
              <svg
                class="h-full w-full"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <defs>
                  <linearGradient id="sri-fill-{uid}-{i}" x1="0" x2="1" y1="0" y2="0">
                    <stop offset="0" stop-color="var(--pr-primary-500)" />
                    <stop offset={fill} stop-color="var(--pr-primary-500)" />
                    <stop offset={fill} stop-color="var(--pr-text-subtle)" />
                    <stop offset="1" stop-color="var(--pr-text-subtle)" />
                  </linearGradient>
                </defs>
                <path d={STAR_PATH} fill="url(#sri-fill-{uid}-{i})" />
              </svg>
            {:else}
              <svg
                class="h-full w-full text-[var(--pr-text-subtle)]"
                viewBox="0 0 24 24"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d={STAR_PATH} />
              </svg>
            {/if}
          </span>
        {/each}
      </span>
      <input
        {id}
        type="range"
        min="1"
        max="5"
        step="0.1"
        bind:value
        class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
        aria-label={label}
        onpointerup={triggerPulse}
        onkeyup={triggerPulse}
      />
    </div>
  </div>
</div>
