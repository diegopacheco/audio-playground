export const TAP_RESET_MS = 2000;
export const TAP_KEEP = 8;
export const MIN_BPM = 20;
export const MAX_BPM = 300;

export function addTap(taps: number[], now: number): number[] {
  const last = taps[taps.length - 1];
  if (last !== undefined && now - last > TAP_RESET_MS) return [now];
  return [...taps, now].slice(-TAP_KEEP);
}

export function bpmFromTaps(taps: number[]): number | null {
  if (taps.length < 2) return null;
  const first = taps[0] ?? 0;
  const last = taps[taps.length - 1] ?? 0;
  const avgMs = (last - first) / (taps.length - 1);
  const bpm = Math.round(600000 / avgMs) / 10;
  if (bpm < MIN_BPM || bpm > MAX_BPM) return null;
  return bpm;
}
