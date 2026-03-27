import { readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const __dirname = dirname(fileURLToPath(import.meta.url));
const faviconPath = join(__dirname, '../src/lib/assets/favicon.svg');
const faviconIcoPath = join(__dirname, '../static/favicon.ico');

describe('favicon.svg', () => {
  it('uses Pocket Ratings branding, not the default Svelte placeholder', () => {
    const svg = readFileSync(faviconPath, 'utf8');
    expect(svg).toContain('<title>Pocket Ratings</title>');
    expect(svg).not.toContain('svelte-logo');
  });
});

describe('static favicon.ico', () => {
  it('exists for clients that default-request /favicon.ico', () => {
    const st = statSync(faviconIcoPath);
    expect(st.isFile()).toBe(true);
    expect(st.size).toBeGreaterThan(0);
    const header = readFileSync(faviconIcoPath).subarray(0, 4);
    // ICO: reserved 0, type 1 (icon)
    expect(header[0]).toBe(0);
    expect(header[1]).toBe(0);
    expect(header[2]).toBe(1);
    expect(header[3]).toBe(0);
  });
});
