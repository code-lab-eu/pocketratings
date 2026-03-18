import { describe, expect, it } from 'vitest';
import {
  errorMessage,
  formatDate,
  formatProductDisplayName,
  formatRating,
  formatVariationDisplay
} from './formatters';

describe('formatRating', () => {
  it('formats whole number to one decimal', () => {
    expect(formatRating(4)).toBe('4.0');
    expect(formatRating(5)).toBe('5.0');
    expect(formatRating(1)).toBe('1.0');
  });

  it('formats one-decimal rating unchanged', () => {
    expect(formatRating(3.8)).toBe('3.8');
    expect(formatRating(4.5)).toBe('4.5');
  });
});

describe('formatVariationDisplay', () => {
  it('returns label when set', () => {
    expect(formatVariationDisplay({ label: '500 g', unit: 'grams' })).toBe('500 g');
  });

  it('returns Default when unit is none and label is empty', () => {
    expect(formatVariationDisplay({ label: '', unit: 'none' })).toBe('Default');
  });

  it('returns capitalized unit when label is empty', () => {
    expect(formatVariationDisplay({ label: '', unit: 'grams' })).toBe('Grams');
    expect(formatVariationDisplay({ label: '', unit: 'milliliters' })).toBe('Milliliters');
    expect(formatVariationDisplay({ label: '', unit: 'other' })).toBe('Other');
  });

  it('passes through unknown unit as-is', () => {
    expect(formatVariationDisplay({ label: '', unit: 'liters' })).toBe('liters');
  });

  it('trims whitespace-only label and falls back to unit', () => {
    expect(formatVariationDisplay({ label: '  ', unit: 'grams' })).toBe('Grams');
  });
});

describe('errorMessage', () => {
  it('returns message from Error instances', () => {
    expect(errorMessage(new Error('Something went wrong.'))).toBe('Something went wrong.');
  });

  it('converts non-Error values to string', () => {
    expect(errorMessage('plain string')).toBe('plain string');
    expect(errorMessage(42)).toBe('42');
    expect(errorMessage(null)).toBe('null');
    expect(errorMessage(undefined)).toBe('undefined');
  });
});

describe('formatProductDisplayName', () => {
  it('returns name with brand when brand is set', () => {
    expect(formatProductDisplayName({ name: 'Milk', brand: 'Acme' })).toBe('Milk \u2014 Acme');
  });

  it('returns name only when brand is empty', () => {
    expect(formatProductDisplayName({ name: 'Milk', brand: '' })).toBe('Milk');
  });

  it('returns name only when brand is absent', () => {
    expect(formatProductDisplayName({ name: 'Milk' })).toBe('Milk');
  });
});

describe('formatDate', () => {
  it('formats a unix timestamp to a locale date string', () => {
    const ts = 1700000000;
    const result = formatDate(ts);
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });

  it('uses provided locale options', () => {
    const ts = 1700000000;
    const short = formatDate(ts);
    const long = formatDate(ts, { year: 'numeric', month: 'long', day: 'numeric' });
    expect(typeof long).toBe('string');
    expect(long.length).toBeGreaterThanOrEqual(short.length);
  });
});
