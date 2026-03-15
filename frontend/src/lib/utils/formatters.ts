/**
 * Rating 1–5 formatted to one decimal place (e.g. "3.8", "4.0").
 */
export function formatRating(r: number): string {
  return Number(r).toFixed(1);
}

/**
 * Display string for a product variation (label/unit) in lists and purchase history.
 * Use label when set; otherwise show unit or "Default" when unit is "none".
 */
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
  const unitLabels: Record<string, string> = {
    grams: 'Grams',
    milliliters: 'Milliliters',
    other: 'Other',
    none: 'Default'
  };
  return unitLabels[v.unit] ?? v.unit;
}
