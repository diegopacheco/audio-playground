import type { Waveform } from "../types";

export const FRAME_SIZE = 2048;
const TONE_GAIN = 0.15;

export interface Frame {
  samples: number[];
  sampleRate: number;
}

export class AudioEngine {
  private context: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private oscillator: OscillatorNode | null = null;
  private gain: GainNode | null = null;
  private micSource: MediaStreamAudioSourceNode | null = null;
  private stream: MediaStream | null = null;

  private ensureContext(): { context: AudioContext; analyser: AnalyserNode } {
    if (!this.context || !this.analyser) {
      this.context = new AudioContext();
      this.analyser = this.context.createAnalyser();
      this.analyser.fftSize = FRAME_SIZE;
    }
    return { context: this.context, analyser: this.analyser };
  }

  async startMic(): Promise<void> {
    this.stop();
    const { context, analyser } = this.ensureContext();
    await context.resume();
    this.stream = await navigator.mediaDevices.getUserMedia({
      audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false },
    });
    this.micSource = context.createMediaStreamSource(this.stream);
    this.micSource.connect(analyser);
  }

  async startTone(frequency: number, waveform: Waveform): Promise<void> {
    this.stop();
    const { context, analyser } = this.ensureContext();
    await context.resume();
    this.oscillator = context.createOscillator();
    this.gain = context.createGain();
    this.gain.gain.value = TONE_GAIN;
    this.oscillator.connect(this.gain);
    this.gain.connect(analyser);
    this.gain.connect(context.destination);
    this.updateTone(frequency, waveform);
    this.oscillator.start();
  }

  updateTone(frequency: number, waveform: Waveform): void {
    if (!this.oscillator || !this.context) {
      return;
    }
    this.oscillator.type = waveform;
    this.oscillator.frequency.setValueAtTime(frequency, this.context.currentTime);
  }

  stop(): void {
    this.oscillator?.stop();
    this.oscillator?.disconnect();
    this.gain?.disconnect();
    this.micSource?.disconnect();
    this.stream?.getTracks().forEach((track) => track.stop());
    this.oscillator = null;
    this.gain = null;
    this.micSource = null;
    this.stream = null;
  }

  frame(): Frame | null {
    if (!this.context || !this.analyser) {
      return null;
    }
    const buffer = new Float32Array(FRAME_SIZE);
    this.analyser.getFloatTimeDomainData(buffer);
    return { samples: Array.from(buffer), sampleRate: this.context.sampleRate };
  }
}
