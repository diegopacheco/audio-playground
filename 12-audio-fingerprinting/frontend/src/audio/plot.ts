import type { HistogramBin } from "../types";

export function scale(value: number, min: number, max: number, size: number): number {
  if (max <= min) {
    return 0;
  }
  return ((value - min) / (max - min)) * size;
}

export interface Column {
  count: number;
  offset: number;
}

export function toColumns(bins: HistogramBin[], width: number): Column[] {
  const columns: Column[] = Array.from({ length: width }, () => ({ count: 0, offset: 0 }));
  if (bins.length === 0 || width <= 0) {
    return columns;
  }
  const min = bins[0].offset_sec;
  const max = bins[bins.length - 1].offset_sec;
  for (const bin of bins) {
    const index = Math.min(width - 1, Math.floor(scale(bin.offset_sec, min, max, width)));
    if (bin.count > columns[index].count) {
      columns[index] = { count: bin.count, offset: bin.offset_sec };
    }
  }
  return columns;
}
