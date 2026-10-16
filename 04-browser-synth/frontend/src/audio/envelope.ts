import type { Adsr } from "../types";

const MIN_RAMP = 0.003;

function heldLevel(adsr: Adsr, t: number): number {
  if (t < adsr.attack) return t / adsr.attack;
  if (t < adsr.attack + adsr.decay) {
    return 1 - ((1 - adsr.sustain) * (t - adsr.attack)) / adsr.decay;
  }
  return adsr.sustain;
}

export function levelAt(adsr: Adsr, t: number, gate = Infinity): number {
  if (t < 0) return 0;
  if (t < gate) return heldLevel(adsr, t);
  const start = heldLevel(adsr, gate);
  const elapsed = t - gate;
  return elapsed >= adsr.release ? 0 : start * (1 - elapsed / adsr.release);
}

export function scheduleAttack(param: AudioParam, adsr: Adsr, startTime: number, peak: number): void {
  const attackEnd = startTime + Math.max(adsr.attack, MIN_RAMP);
  param.cancelScheduledValues(startTime);
  param.setValueAtTime(0, startTime);
  param.linearRampToValueAtTime(peak, attackEnd);
  param.linearRampToValueAtTime(peak * adsr.sustain, attackEnd + Math.max(adsr.decay, MIN_RAMP));
}

export function scheduleRelease(
  param: AudioParam,
  adsr: Adsr,
  startTime: number,
  releaseTime: number,
  peak: number,
): number {
  const current = peak * levelAt(adsr, releaseTime - startTime);
  const end = releaseTime + Math.max(adsr.release, MIN_RAMP);
  param.cancelScheduledValues(releaseTime);
  param.setValueAtTime(current, releaseTime);
  param.linearRampToValueAtTime(0, end);
  return end;
}
