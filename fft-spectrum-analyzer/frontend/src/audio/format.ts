export function formatHz(hz: number): string {
  if (hz >= 1000) return `${(hz / 1000).toFixed(2)} kHz`;
  if (hz < 100) return `${hz.toFixed(1)} Hz`;
  return `${Math.round(hz)} Hz`;
}

export function formatDb(db: number): string {
  return `${db.toFixed(1)} dB`;
}
