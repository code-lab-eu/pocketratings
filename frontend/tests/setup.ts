import '@testing-library/jest-dom/vitest';

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
