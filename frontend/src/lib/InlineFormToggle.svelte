<script lang="ts">
  import type { Snippet } from 'svelte';
  import { slide } from 'svelte/transition';
  import Button from '$lib/Button.svelte';
  import { inlineFormSlideParams } from '$lib/inlineFormMotion';

  interface Props {
    /** Text of the opening button and of the form heading. */
    label: string;
    /** Id of the form heading, for the form's aria-labelledby. */
    headingId: string;
    /** Whether the form is shown instead of the button. */
    open: boolean;
    /** Called when the closing transition has finished (e.g. to reset form state). */
    onoutroend?: () => void;
    children: Snippet;
  }

  let { label, headingId, open = $bindable(), onoutroend, children }: Props = $props();
</script>

{#if !open}
  <p class="mt-4">
    <Button variant="link" onclick={() => (open = true)}>
      {label}
    </Button>
  </p>
{:else}
  <div
    class="mt-4 pr-inline-form"
    in:slide={inlineFormSlideParams()}
    out:slide={inlineFormSlideParams()}
    {onoutroend}
  >
    <h3 id={headingId} class="mb-3 text-base font-semibold pr-text-body">
      {label}
    </h3>
    {@render children()}
  </div>
{/if}
