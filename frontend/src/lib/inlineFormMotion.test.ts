import { describe, expect, it, vi, afterEach } from 'vitest';
import { INLINE_FORM_TRANSITION_MS, inlineFormMotionMs, inlineFormSlideParams } from './inlineFormMotion';

describe('inlineFormMotion', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('inlineFormMotionMs returns base when matchMedia is missing', () => {
    vi.stubGlobal('window', {});
    expect(inlineFormMotionMs(280)).toBe(280);
  });

  it('inlineFormMotionMs returns base when reduced motion is off', () => {
    vi.stubGlobal('window', {
      matchMedia: () => ({
        matches: false,
        media: '',
        addEventListener: vi.fn(),
        removeEventListener: vi.fn()
      })
    });
    expect(inlineFormMotionMs(280)).toBe(280);
  });

  it('inlineFormMotionMs returns 0 when prefers-reduced-motion matches', () => {
    vi.stubGlobal('window', {
      matchMedia: () => ({
        matches: true,
        media: '(prefers-reduced-motion: reduce)',
        addEventListener: vi.fn(),
        removeEventListener: vi.fn()
      })
    });
    expect(inlineFormMotionMs(280)).toBe(0);
  });

  it('inlineFormSlideParams uses axis y and duration from inlineFormMotionMs', () => {
    vi.stubGlobal('window', {
      matchMedia: () => ({
        matches: false,
        media: '',
        addEventListener: vi.fn(),
        removeEventListener: vi.fn()
      })
    });
    const p = inlineFormSlideParams();
    expect(p.axis).toBe('y');
    expect(p.duration).toBe(INLINE_FORM_TRANSITION_MS);
  });
});
