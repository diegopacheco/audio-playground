import type { Capture, SourceKind } from "../types";

const TAP_SOURCE = `
class PcmTap extends AudioWorkletProcessor {
  process(inputs) {
    const channel = inputs[0] && inputs[0][0];
    if (channel) this.port.postMessage(channel.slice(0));
    return true;
  }
}
registerProcessor("pcm-tap", PcmTap);
`;

const TONE_GAIN = 0.5;
const FALLBACK_BUFFER = 2048;

async function workletTap(ctx: AudioContext, onChunk: (chunk: Float32Array) => void): Promise<AudioNode> {
  const url = URL.createObjectURL(new Blob([TAP_SOURCE], { type: "application/javascript" }));
  try {
    await ctx.audioWorklet.addModule(url);
  } finally {
    URL.revokeObjectURL(url);
  }
  const node = new AudioWorkletNode(ctx, "pcm-tap");
  node.port.onmessage = (event: MessageEvent<Float32Array>) => onChunk(event.data);
  return node;
}

function scriptTap(ctx: AudioContext, onChunk: (chunk: Float32Array) => void): AudioNode {
  const node = ctx.createScriptProcessor(FALLBACK_BUFFER, 1, 1);
  node.onaudioprocess = (event: AudioProcessingEvent) => {
    onChunk(event.inputBuffer.getChannelData(0).slice(0));
  };
  return node;
}

async function createTap(ctx: AudioContext, onChunk: (chunk: Float32Array) => void): Promise<AudioNode> {
  if (ctx.audioWorklet) return workletTap(ctx, onChunk);
  return scriptTap(ctx, onChunk);
}

export async function startCapture(
  source: SourceKind,
  toneHz: number,
  onChunk: (chunk: Float32Array) => void,
): Promise<Capture> {
  const ctx = new AudioContext();
  const tap = await createTap(ctx, onChunk);
  const mute = ctx.createGain();
  mute.gain.value = 0;
  tap.connect(mute).connect(ctx.destination);

  let oscillator: OscillatorNode | null = null;
  let stream: MediaStream | null = null;

  if (source === "tone") {
    oscillator = ctx.createOscillator();
    oscillator.frequency.value = toneHz;
    const gain = ctx.createGain();
    gain.gain.value = TONE_GAIN;
    oscillator.connect(gain).connect(tap);
    oscillator.start();
  } else {
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false },
      });
    } catch (e) {
      await ctx.close();
      throw e;
    }
    ctx.createMediaStreamSource(stream).connect(tap);
  }

  await ctx.resume();

  return {
    sampleRate: ctx.sampleRate,
    setToneHz: (hz: number) => {
      if (oscillator) oscillator.frequency.setValueAtTime(hz, ctx.currentTime);
    },
    stop: async () => {
      oscillator?.stop();
      stream?.getTracks().forEach((t) => t.stop());
      await ctx.close();
    },
  };
}
