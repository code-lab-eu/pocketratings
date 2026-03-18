/**
 * Rating 1-5 formatted to one decimal place (e.g. "3.8", "4.0").
 */
export function formatRating(r: number): string {
  return Number(r).toFixed(1);
}

/**
 * Display string for a product variation (label/unit) in lists and purchase history.
 * Use label when set; otherwise show unit or "Default" when unit is "none".
 */
const unitLabels: Record<string, string> = {
  grams: 'Grams',
  milliliters: 'Milliliters',
  other: 'Other'
};

export function formatVariationDisplay(v: {
  label: string;
  unit: string;
  quantity?: number | null;
}): string {
  if (v.label.trim() !== '') {
    return v.label.trim();
  }
  if (v.unit === 'none') {
    return 'Default';
  }
  return unitLabels[v.unit] ?? v.unit;
}

export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function formatProductDisplayName(p: { name: string; brand?: string }): string {
  return p.brand ? `${p.name} \u2014 ${p.brand}` : p.name;
}

export const STAR_SVG_PATH =
  'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z';

export function formatDate(
  unixSeconds: number,
  options?: Intl.DateTimeFormatOptions
): string {
  return new Date(unixSeconds * 1000).toLocaleDateString(
    undefined,
    options ?? { year: 'numeric', month: 'short', day: 'numeric' }
  );
}
