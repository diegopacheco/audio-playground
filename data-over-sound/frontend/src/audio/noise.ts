export type Random = () => number;

export function seededRandom(seed: number): Random {
  let state = seed >>> 0 || 1;
  return () => {
    state ^= state << 13;
    state >>>= 0;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return (state + 1) / 4294967297;
  };
}

function gaussian(random: Random): number {
  return Math.sqrt(-2 * Math.log(random())) * Math.cos(2 * Math.PI * random());
}

export function signalPower(samples: Float32Array): number {
  let sum = 0;
  let count = 0;
  for (const s of samples) {
    if (s !== 0) {
      sum += s * s;
      count++;
    }
  }
  return count === 0 ? 0 : sum / count;
}

export function addWhiteNoise(samples: Float32Array, snrDb: number, random: Random): Float32Array {
  const sigma = Math.sqrt(signalPower(samples) / 10 ** (snrDb / 10));
  const out = new Float32Array(samples.length);
  for (let i = 0; i < samples.length; i++) {
    out[i] = samples[i] + gaussian(random) * sigma;
  }
  return out;
}

export function withLeadingSilence(samples: Float32Array, count: number): Float32Array {
  const out = new Float32Array(samples.length + count);
  out.set(samples, count);
  return out;
}
