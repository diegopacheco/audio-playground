export interface Recording {
  samples: Float32Array;
  rate: number;
}

export async function recordMic(seconds: number): Promise<Recording> {
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false },
  });
  const recorder = new MediaRecorder(stream);
  const chunks: Blob[] = [];
  recorder.ondataavailable = (event) => chunks.push(event.data);
  const stopped = new Promise<void>((resolve) => {
    recorder.onstop = () => resolve();
  });
  recorder.start();
  await new Promise((resolve) => setTimeout(resolve, seconds * 1000));
  recorder.stop();
  await stopped;
  stream.getTracks().forEach((track) => track.stop());
  const context = new AudioContext();
  try {
    const decoded = await context.decodeAudioData(await new Blob(chunks).arrayBuffer());
    return { samples: decoded.getChannelData(0).slice(), rate: decoded.sampleRate };
  } finally {
    await context.close();
  }
}
