/** Duration (ms) for `slide` transitions on expand-in-place inline forms. */
export const INLINE_FORM_TRANSITION_MS = 280;

/** Respects `prefers-reduced-motion` for Svelte transition `duration`. */
export function inlineFormMotionMs(baseMs: number): number {
  if (typeof globalThis.window === 'undefined') {
    return baseMs;
  }
  const { matchMedia } = globalThis.window;
  if (typeof matchMedia !== 'function') {
    return baseMs;
  }
  return matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : baseMs;
}

/** Params for `import { slide } from 'svelte/transition'` on inline form shells. */
export function inlineFormSlideParams(): { duration: number; axis: 'y' } {
  return {
    duration: inlineFormMotionMs(INLINE_FORM_TRANSITION_MS),
    axis: 'y'
  };
}
