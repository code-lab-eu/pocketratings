<script lang="ts">
  interface Props {
    /** True when the app is in dark mode (sun shown to switch to light). */
    dark: boolean;
  }

  let { dark }: Props = $props();
</script>

<span
  class="pr-theme-toggle"
  aria-hidden="true"
  data-active={dark ? 'sun' : 'moon'}
>
  <span class="pr-theme-toggle__rotor">
    <svg
      class="pr-theme-toggle__sun"
      viewBox="0 0 24 24"
      width="100%"
      height="100%"
      aria-hidden="true"
    >
      <circle class="pr-theme-toggle__disc" cx="12" cy="12" r="3.5" fill="currentColor" />
      <g
        class="pr-theme-toggle__rays"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      >
        {#each [0, 45, 90, 135, 180, 225, 270, 315] as deg (deg)}
          {@const rad = (deg * Math.PI) / 180}
          {@const x1 = 12 + 5 * Math.cos(rad)}
          {@const y1 = 12 + 5 * Math.sin(rad)}
          {@const x2 = 12 + 8.25 * Math.cos(rad)}
          {@const y2 = 12 + 8.25 * Math.sin(rad)}
          <line x1={x1} y1={y1} x2={x2} y2={y2} />
        {/each}
      </g>
    </svg>
    <span class="pr-theme-toggle__moon-wrap">
      <svg
        class="pr-theme-toggle__moon"
        viewBox="0 0 24 24"
        width="100%"
        height="100%"
        aria-hidden="true"
      >
        <path
          fill="currentColor"
          d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"
        />
      </svg>
    </span>
  </span>
</span>
