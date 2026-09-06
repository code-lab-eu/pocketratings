import '@testing-library/jest-dom/vitest';

// Node 26 enables the Web Storage API by default, so `localStorage` and
// `sessionStorage` already exist as (non-functional, file-backed) globals.
// Vitest's jsdom environment skips any window key that is already present on
// the Node global, so it never installs jsdom's working Storage objects and
// app code reading the bare global gets Node's stub instead. Point the globals
// at the jsdom window's storage. Remove once we are on a Vitest release that
// overrides these itself (fixed in Vitest 5).
const jsdomWindow = (globalThis as { jsdom?: { window: Window } }).jsdom?.window;
if (jsdomWindow) {
  for (const key of ['localStorage', 'sessionStorage'] as const) {
    Object.defineProperty(globalThis, key, {
      value: jsdomWindow[key],
      configurable: true,
      enumerable: true,
      writable: true
    });
  }
}

if (typeof Element.prototype.animate !== 'function') {
  // Web Animations API stub for JSDOM (Svelte 5 transition:slide uses element.animate)
  Element.prototype.animate = function () {
    let finishCb: (() => void) | null = null;
    const anim = {
      cancel: () => {},
      finish: () => { if (finishCb) finishCb(); },
      play: () => {},
      pause: () => {},
      reverse: () => {},
      get onfinish() { return finishCb; },
      set onfinish(fn: (() => void) | null) {
        finishCb = fn;
        if (fn) queueMicrotask(() => fn());
      },
      oncancel: null,
      finished: Promise.resolve(null),
      currentTime: 0,
      playState: 'finished',
      effect: null,
      timeline: null,
      startTime: null,
      playbackRate: 1,
      pending: false,
      id: '',
      persist: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
      commitStyles: () => {},
      replaceState: () => {},
      updatePlaybackRate: () => {},
      remove: () => {},
    } as unknown as Animation;
    return anim;
  };
}
