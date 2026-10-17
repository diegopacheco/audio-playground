import captureUrl from "./capture.worklet.ts?worker&url";
import playbackUrl from "./playback.worklet.ts?worker&url";
import { fetchTestSignal, voiceSocketUrl } from "../api";
import { BLOCK_SIZE } from "./constants";
import { RoundTripTracker, SILENCE_DB, rms, samplesToMs, toDb } from "./metrics";
import type { EngineCallbacks, EngineSnapshot, ErrorBody, SessionInfo, VoiceSettings } from "../types";

const TEST_SIGNAL_SECONDS = 6;

export class VoiceEngine {
  private context: AudioContext | null = null;
  private capture: AudioWorkletNode | null = null;
  private playback: AudioWorkletNode | null = null;
  private source: AudioNode | null = null;
  private stream: MediaStream | null = null;
  private socket: WebSocket | null = null;
  private readonly tracker = new RoundTripTracker();
  private latencySamples = 0;
  private bufferedSamples = 0;
  private inputDb = SILENCE_DB;
  private outputDb = SILENCE_DB;

  constructor(private readonly callbacks: EngineCallbacks) {}

  async start(kind: "mic" | "test", settings: VoiceSettings): Promise<void> {
    this.stop();
    const context = await this.ensureContext();
    await context.resume();
    this.source = kind === "mic" ? await this.micSource(context) : await this.testSource(context);
    this.capture = new AudioWorkletNode(context, "capture");
    this.playback = new AudioWorkletNode(context, "playback", { numberOfInputs: 0, outputChannelCount: [1] });
    const mute = context.createGain();
    mute.gain.value = 0;
    this.source.connect(this.capture);
    this.capture.connect(mute);
    mute.connect(context.destination);
    this.playback.connect(context.destination);
    this.capture.port.onmessage = (event: MessageEvent<Float32Array<ArrayBuffer>>) => this.onCaptured(event.data);
    this.playback.port.onmessage = (event: MessageEvent<number>) => {
      this.bufferedSamples = event.data;
    };
    this.connect(settings);
  }

  update(settings: VoiceSettings): void {
    if (this.capture) {
      this.connect(settings);
    }
  }

  stop(): void {
    this.socket?.close();
    this.socket = null;
    if (this.source instanceof AudioBufferSourceNode) {
      this.source.stop();
    }
    this.source?.disconnect();
    this.capture?.disconnect();
    this.playback?.disconnect();
    this.stream?.getTracks().forEach((track) => track.stop());
    this.source = null;
    this.capture = null;
    this.playback = null;
    this.stream = null;
    this.inputDb = SILENCE_DB;
    this.outputDb = SILENCE_DB;
    this.bufferedSamples = 0;
    this.tracker.reset();
  }

  snapshot(): EngineSnapshot {
    const rate = this.context?.sampleRate ?? 0;
    const blockMs = samplesToMs(BLOCK_SIZE, rate);
    const roundTripMs = this.tracker.average;
    const algorithmMs = samplesToMs(this.latencySamples, rate);
    const bufferMs = samplesToMs(this.bufferedSamples, rate);
    return {
      inputDb: this.inputDb,
      outputDb: this.outputDb,
      latency: {
        blockMs,
        roundTripMs,
        algorithmMs,
        bufferMs,
        totalMs: blockMs + roundTripMs + algorithmMs + bufferMs,
      },
    };
  }

  private async ensureContext(): Promise<AudioContext> {
    if (!this.context) {
      const context = new AudioContext();
      await context.audioWorklet.addModule(captureUrl);
      await context.audioWorklet.addModule(playbackUrl);
      this.context = context;
    }
    return this.context;
  }

  private async micSource(context: AudioContext): Promise<AudioNode> {
    this.stream = await navigator.mediaDevices.getUserMedia({
      audio: { echoCancellation: true, noiseSuppression: false, autoGainControl: false, channelCount: 1 },
    });
    return context.createMediaStreamSource(this.stream);
  }

  private async testSource(context: AudioContext): Promise<AudioNode> {
    const wav = await fetchTestSignal(context.sampleRate, TEST_SIGNAL_SECONDS);
    const buffer = await context.decodeAudioData(await wav.arrayBuffer());
    const node = context.createBufferSource();
    node.buffer = buffer;
    node.loop = true;
    node.start();
    return node;
  }

  private connect(settings: VoiceSettings): void {
    const rate = this.context?.sampleRate ?? 48000;
    this.socket?.close();
    this.tracker.reset();
    const socket = new WebSocket(voiceSocketUrl(window.location.origin, settings, rate));
    socket.binaryType = "arraybuffer";
    socket.onmessage = (event: MessageEvent<ArrayBuffer | string>) => this.onMessage(event.data);
    socket.onerror = () => this.callbacks.onError("voice stream connection failed");
    this.socket = socket;
  }

  private onCaptured(block: Float32Array<ArrayBuffer>): void {
    this.inputDb = toDb(rms(block));
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.tracker.sent(performance.now());
      this.socket.send(block.buffer);
    }
  }

  private onMessage(data: ArrayBuffer | string): void {
    if (typeof data === "string") {
      const body = JSON.parse(data) as SessionInfo | ErrorBody;
      if ("error" in body) {
        this.callbacks.onError(body.error);
      } else {
        this.latencySamples = body.latencySamples;
        this.callbacks.onSession(body);
      }
      return;
    }
    this.tracker.received(performance.now());
    const block = new Float32Array(data);
    this.outputDb = toDb(rms(block));
    this.playback?.port.postMessage(block, [block.buffer]);
  }
}
