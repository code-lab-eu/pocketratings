import { describe, expect, it } from 'vitest';
import { formatRating, formatVariationDisplay } from './formatters';

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
});
