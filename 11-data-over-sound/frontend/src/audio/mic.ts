export interface MicCapture {
  sampleRate: number;
  stop: () => void;
}

const BUFFER_SIZE = 4096;

export async function startMic(onFrame: (frame: Float32Array) => void): Promise<MicCapture> {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false },
  });
  const ctx = new AudioContext();
  const source = ctx.createMediaStreamSource(stream);
  const processor = ctx.createScriptProcessor(BUFFER_SIZE, 1, 1);
  const mute = ctx.createGain();
  mute.gain.value = 0;
  processor.onaudioprocess = (e) => onFrame(new Float32Array(e.inputBuffer.getChannelData(0)));
  source.connect(processor);
  processor.connect(mute);
  mute.connect(ctx.destination);
  return {
    sampleRate: ctx.sampleRate,
    stop: () => {
      processor.disconnect();
      source.disconnect();
      stream.getTracks().forEach((t) => t.stop());
      void ctx.close();
    },
  };
}
