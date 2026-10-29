// WCAG AA guard for the design tokens in src/styles/app.css (spec 2.2-2.4, 8).
// Every token meant for readable text must reach 4.5:1 on the app and
// surface backgrounds in both themes (Geist palette). Filled controls are
// the neutral primary button and the blue accent; both must carry their text.
// Danger buttons are outlined, so they're covered by text-danger above.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const css = readFileSync(resolve(__dirname, '../src/styles/app.css'), 'utf8');

function block(selectorStart: RegExp): Record<string, string> {
  const m = selectorStart.exec(css);
  if (!m) throw new Error(`block not found: ${selectorStart}`);
  const body = css.slice(m.index + m[0].length, css.indexOf('}', m.index));
  const out: Record<string, string> = {};
  for (const [, name, value] of body.matchAll(/--ui-([\w-]+):\s*([^;]+);/g)) out[name] = value.trim();
  return out;
}

const themes = {
  dark: block(/:root,\s*\.dark\s*\{/),
  light: block(/\n\.light\s*\{/),
};

function luminance(hex: string): number {
  const h = hex.replace('#', '');
  const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16) / 255);
  const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function contrast(a: string, b: string): number {
  const [l1, l2] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (l1 + 0.05) / (l2 + 0.05);
}

const TEXT_TOKENS = ['primary', 'secondary', 'muted', 'accent-text', 'danger', 'warning', 'success', 'info'];
const BACKGROUNDS = ['app', 'surface'];

describe('token contrast (WCAG AA 4.5:1)', () => {
  for (const [theme, t] of Object.entries(themes)) {
    for (const fg of TEXT_TOKENS) {
      for (const bg of BACKGROUNDS) {
        it(`${theme}: text-${fg} on bg-${bg}`, () => {
          expect(t[fg], `--ui-${fg} missing`).toMatch(/^#/);
          expect(contrast(t[fg], t[bg])).toBeGreaterThanOrEqual(4.5);
        });
      }
    }

    it(`${theme}: on-btn text on the primary button fill`, () => {
      expect(contrast(t['on-btn'], t['btn'])).toBeGreaterThanOrEqual(4.5);
    });

    it(`${theme}: on-accent text on the accent fill`, () => {
      expect(contrast(t['on-accent'], t['accent'])).toBeGreaterThanOrEqual(4.5);
    });

    // Non-text UI (WCAG 1.4.11): dots, rails, glyphs and faint icons need 3:1.
    for (const tok of ['danger-solid', 'warning-solid', 'success-solid', 'info-solid', 'faint', 'danger', 'warning']) {
      for (const bg of BACKGROUNDS) {
        it(`${theme}: ${tok} on bg-${bg} (3:1, non-text)`, () => {
          expect(t[tok], `--ui-${tok} missing`).toMatch(/^#/);
          expect(contrast(t[tok], t[bg])).toBeGreaterThanOrEqual(3);
        });
      }
    }

    it(`${theme}: no tinted status fills`, () => {
      expect(t['tint']).toBe('0%');
    });
  }
});

describe('legacy text aliases', () => {
  it('no view uses the fill-strength accent/status aliases for text', () => {
    // These alias to --ui-accent / --ui-*-solid, which are 3:1 fills, not text colours.
    const files = import.meta.glob('../src/**/*.{svelte,ts}', { query: '?raw', import: 'default', eager: true });
    const offenders = Object.entries(files)
      .filter(([, src]) => /(?<![\w-])text-accent-(primary|secondary|warning|success|error)(?![\w-])/.test(src as string))
      .map(([p]) => p);
    expect(offenders).toEqual([]);
  });
});
