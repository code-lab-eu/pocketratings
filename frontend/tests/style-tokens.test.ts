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
});
