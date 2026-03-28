import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

const __dirname = dirname(fileURLToPath(import.meta.url));
const layoutCssPath = join(__dirname, '..', 'src', 'routes', 'layout.css');

function loadLayoutCss(): string {
  return readFileSync(layoutCssPath, 'utf-8');
}

describe('layout.css design tokens and focus states', () => {
  it('defines focus ring tokens in :root', () => {
    const css = loadLayoutCss();
    expect(css).toContain('--pr-focus-ring-color:');
    expect(css).toContain('--pr-focus-ring-width:');
    expect(css).toContain('--pr-focus-ring-offset:');
  });

  it('defines focus ring tokens in html.dark', () => {
    const css = loadLayoutCss();
    const darkBlock = css.includes('html.dark')
      ? css.slice(css.indexOf('html.dark'))
      : '';
    expect(darkBlock).toContain('--pr-focus-ring-color:');
    expect(darkBlock).toContain('--pr-focus-ring-width:');
    expect(darkBlock).toContain('--pr-focus-ring-offset:');
  });

  it('uses focus tokens in .pr-btn-primary:focus-visible', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-btn-primary:focus-visible\s*\{[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color[\s\S]*?--pr-focus-ring-offset/
    );
  });

  it('uses focus tokens in .pr-btn-secondary:focus-visible', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-btn-secondary:focus-visible\s*\{[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color[\s\S]*?--pr-focus-ring-offset/
    );
  });

  it('gives .pr-card:focus-visible a visible outline using focus tokens', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-card:focus-visible\s*\{[\s\S]*?outline:[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color/
    );
  });

  it('gives .pr-btn-primary cursor: pointer', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-btn-primary\s*\{[\s\S]*?cursor:\s*pointer/
    );
  });

  it('gives .pr-btn-secondary cursor: pointer', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-btn-secondary\s*\{[\s\S]*?cursor:\s*pointer/
    );
  });

  it('gives .pr-card cursor: pointer', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-card\s*\{[\s\S]*?cursor:\s*pointer/
    );
  });

  it('gives .pr-link-muted cursor: pointer and focus-visible using tokens', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-link-muted\s*\{[\s\S]*?cursor:\s*pointer/
    );
    expect(css).toMatch(
      /\.pr-link-muted:focus-visible\s*\{[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color[\s\S]*?--pr-focus-ring-offset/
    );
  });

  it('gives .pr-link-inline cursor: pointer and focus-visible using tokens', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-link-inline\s*\{[\s\S]*?cursor:\s*pointer/
    );
    expect(css).toMatch(
      /\.pr-link-inline:focus-visible\s*\{[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color[\s\S]*?--pr-focus-ring-offset/
    );
  });

  it('gives .pr-btn-icon cursor: pointer and focus-visible using tokens', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-btn-icon\s*\{[\s\S]*?cursor:\s*pointer/
    );
    expect(css).toMatch(
      /\.pr-btn-icon:focus-visible\s*\{[\s\S]*?--pr-focus-ring-width[\s\S]*?--pr-focus-ring-color[\s\S]*?--pr-focus-ring-offset/
    );
  });

  it('uses an enlarged font size for page title headings', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-heading-page\s*\{[\s\S]*?font-size:\s*1\.75rem/
    );
  });

  it('defines Inter and Fraunces stacks in :root', () => {
    const css = loadLayoutCss();
    const rootBlock = css.slice(css.indexOf(':root'), css.indexOf('html.dark'));
    expect(rootBlock).toContain("--pr-font-sans: 'Inter'");
    expect(rootBlock).toContain("--pr-font-display: 'Fraunces'");
  });

  it('applies display font token to section headings', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-heading-section\s*\{[\s\S]*?font-family:\s*var\(--pr-font-display\)/
    );
  });

  it('defines theme toggle animation with ~400ms transitions', () => {
    const css = loadLayoutCss();
    expect(css).toContain('.pr-theme-toggle');
    expect(css).toMatch(/\.pr-theme-toggle[\s\S]*?400ms/);
  });

  it('disables theme toggle motion under prefers-reduced-motion', () => {
    const css = loadLayoutCss();
    const idx = css.indexOf('@media (prefers-reduced-motion: reduce)');
    expect(idx).toBeGreaterThanOrEqual(0);
    const fromReduce = css.slice(idx);
    expect(fromReduce).toContain('.pr-theme-toggle');
  });

  it('rotates sun rays around the center when sun shows or hides', () => {
    const css = loadLayoutCss();
    expect(css).toMatch(
      /\.pr-theme-toggle\[data-active='sun'\] \.pr-theme-toggle__rays\s*\{[\s\S]*?rotate\(0deg\)/
    );
    expect(css).toMatch(
      /\.pr-theme-toggle\[data-active='moon'\] \.pr-theme-toggle__rays\s*\{[\s\S]*?rotate\(90deg\)/
    );
  });
});
