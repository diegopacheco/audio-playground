export const SILENCE_DB = -60;

export function rms(samples: Float32Array): number {
  if (samples.length === 0) {
    return 0;
  }
  let sum = 0;
  for (const sample of samples) {
    sum += sample * sample;
  }
  return Math.sqrt(sum / samples.length);
}

export function toDb(level: number): number {
  if (level <= 0) {
    return SILENCE_DB;
  }
  return Math.max(SILENCE_DB, 20 * Math.log10(level));
}

export function meterPercent(db: number): number {
  return Math.min(100, Math.max(0, ((db - SILENCE_DB) / -SILENCE_DB) * 100));
}

export function samplesToMs(samples: number, rate: number): number {
  return rate > 0 ? (samples / rate) * 1000 : 0;
}

export class RoundTripTracker {
  private pending: number[] = [];
  private smoothed = 0;

  constructor(private readonly smoothing = 0.1) {}

  sent(now: number): void {
    this.pending.push(now);
  }

  received(now: number): number {
    const sentAt = this.pending.shift();
    if (sentAt === undefined) {
      return this.smoothed;
    }
    const sample = now - sentAt;
    this.smoothed = this.smoothed === 0 ? sample : this.smoothed + (sample - this.smoothed) * this.smoothing;
    return this.smoothed;
  }

  get average(): number {
    return this.smoothed;
  }

  reset(): void {
    this.pending = [];
    this.smoothed = 0;
  }
}
