const NORMAL_HZ = 1000;
const ACCENT_HZ = 1600;
const DURATION_S = 0.03;

export function playClick(ctx: AudioContext, time: number, accent: boolean): void {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.frequency.value = accent ? ACCENT_HZ : NORMAL_HZ;
  gain.gain.setValueAtTime(accent ? 0.6 : 0.4, time);
  gain.gain.exponentialRampToValueAtTime(0.001, time + DURATION_S);
  osc.connect(gain).connect(ctx.destination);
  osc.start(time);
  osc.stop(time + DURATION_S);
}
