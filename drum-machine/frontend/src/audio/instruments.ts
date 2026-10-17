import type { InstrumentName, Voice } from "../types";

const noiseBuffers = new WeakMap<BaseAudioContext, AudioBuffer>();

function noiseBuffer(ctx: AudioContext): AudioBuffer {
  const cached = noiseBuffers.get(ctx);
  if (cached) return cached;
  const buffer = ctx.createBuffer(1, ctx.sampleRate, ctx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;
  noiseBuffers.set(ctx, buffer);
  return buffer;
}

function decayGain(ctx: AudioContext, time: number, peak: number, tau: number): GainNode {
  const gain = ctx.createGain();
  gain.gain.setValueAtTime(peak, time);
  gain.gain.setTargetAtTime(0.0001, time, tau);
  return gain;
}

function noise(ctx: AudioContext, time: number, length: number): AudioBufferSourceNode {
  const src = ctx.createBufferSource();
  src.buffer = noiseBuffer(ctx);
  src.start(time);
  src.stop(time + length);
  return src;
}

const kick: Voice = (ctx, out, time) => {
  const osc = ctx.createOscillator();
  osc.frequency.setValueAtTime(150, time);
  osc.frequency.setTargetAtTime(45, time, 0.03);
  const amp = decayGain(ctx, time, 0.95, 0.12);
  osc.connect(amp).connect(out);
  osc.start(time);
  osc.stop(time + 0.6);
};

const snare: Voice = (ctx, out, time) => {
  const band = ctx.createBiquadFilter();
  band.type = "bandpass";
  band.frequency.value = 1800;
  band.Q.value = 0.8;
  noise(ctx, time, 0.3).connect(band).connect(decayGain(ctx, time, 0.9, 0.05)).connect(out);
  const tone = ctx.createOscillator();
  tone.frequency.value = 185;
  tone.connect(decayGain(ctx, time, 0.35, 0.04)).connect(out);
  tone.start(time);
  tone.stop(time + 0.3);
};

const hihat: Voice = (ctx, out, time) => {
  const hp = ctx.createBiquadFilter();
  hp.type = "highpass";
  hp.frequency.value = 7000;
  noise(ctx, time, 0.12).connect(hp).connect(decayGain(ctx, time, 0.5, 0.018)).connect(out);
};

const clap: Voice = (ctx, out, time) => {
  const band = ctx.createBiquadFilter();
  band.type = "bandpass";
  band.frequency.value = 1200;
  band.Q.value = 1.2;
  const amp = ctx.createGain();
  amp.gain.setValueAtTime(0, time);
  for (let i = 0; i < 3; i++) {
    const t = time + i * 0.011;
    amp.gain.setValueAtTime(1, t);
    amp.gain.setTargetAtTime(0.05, t, 0.004);
  }
  const tail = time + 0.033;
  amp.gain.setValueAtTime(0.8, tail);
  amp.gain.setTargetAtTime(0.0001, tail, 0.06);
  noise(ctx, time, 0.35).connect(band).connect(amp).connect(out);
};

export const VOICES: Record<InstrumentName, Voice> = { kick, snare, hihat, clap };
