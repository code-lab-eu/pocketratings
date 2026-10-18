import { afterAll, beforeAll } from 'vitest';

/**
 * Runs the tests in the calling file in the given IANA timezone. Node applies a
 * change to `process.env.TZ` immediately, so local-time `Date` methods follow it.
 */
export function pinTimezone(tz: string): void {
  const original = process.env.TZ;

  beforeAll(() => {
    process.env.TZ = tz;
  });

  afterAll(() => {
    if (original === undefined) {
      delete process.env.TZ;
    } else {
      process.env.TZ = original;
    }
  });
}
