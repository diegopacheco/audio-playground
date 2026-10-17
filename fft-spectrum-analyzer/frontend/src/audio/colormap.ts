import type { Rgb } from "../types";

export const MIN_DB = -100;
export const MAX_DB = 0;

const STOPS: Rgb[] = [
  [250, 250, 252],
  [186, 214, 245],
  [92, 141, 222],
  [118, 72, 196],
  [232, 96, 74],
  [252, 196, 64],
];

export function dbToUnit(db: number): number {
  const t = (db - MIN_DB) / (MAX_DB - MIN_DB);
  return Math.min(1, Math.max(0, t));
}

export function heatColor(unit: number): Rgb {
  const t = Math.min(1, Math.max(0, unit)) * (STOPS.length - 1);
  const i = Math.min(STOPS.length - 2, Math.floor(t));
  const f = t - i;
  const a = STOPS[i];
  const b = STOPS[i + 1];
  return [0, 1, 2].map((c) => Math.round(a[c] + (b[c] - a[c]) * f)) as Rgb;
}

export function rgbCss(rgb: Rgb): string {
  return `rgb(${rgb[0]}, ${rgb[1]}, ${rgb[2]})`;
}
