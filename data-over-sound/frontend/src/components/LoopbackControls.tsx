import { useState } from "react";
import { demodulate } from "../api";
import { addWhiteNoise, seededRandom, withLeadingSilence } from "../audio/noise";
import type { DemodResult, ModemSettings, PcmAudio } from "../types";
import type { Receiver } from "../useReceiver";
import { Waveform } from "./Waveform";

interface Props {
  settings: ModemSettings;
  sent: PcmAudio | null;
  receiver: Receiver;
}

const CHUNK = 2048;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function LoopbackControls({ settings, sent, receiver }: Props) {
  const [snr, setSnr] = useState(10);
  const [silenceMs, setSilenceMs] = useState(250);
  const [noisy, setNoisy] = useState<Float32Array | null>(null);
  const [result, setResult] = useState<DemodResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  function channel(audio: PcmAudio): Float32Array {
    const padded = withLeadingSilence(audio.samples, Math.round((silenceMs / 1000) * audio.sampleRate));
    const out = addWhiteNoise(padded, snr, seededRandom(Date.now()));
    setNoisy(out);
    return out;
  }

  async function stream() {
    if (!sent) return;
    setBusy(true);
    setError(null);
    try {
      const samples = channel(sent);
      await receiver.open(sent.sampleRate);
      const chunkMs = (CHUNK / sent.sampleRate) * 1000;
      for (let i = 0; i < samples.length; i += CHUNK) {
        receiver.send(samples.subarray(i, i + CHUNK));
        await sleep(chunkMs);
      }
      await sleep(300);
      receiver.close();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function decodeWhole() {
    if (!sent) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(await demodulate(channel(sent), sent.sampleRate, settings));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="controls">
      <div className="row">
        <label className="slider">
          Channel SNR <b>{snr} dB</b>
          <input type="range" min={-6} max={30} value={snr} onChange={(e) => setSnr(Number(e.target.value))} />
        </label>
        <label className="slider">
          Leading silence <b>{silenceMs} ms</b>
          <input
            type="range"
            min={0}
            max={2000}
            step={50}
            value={silenceMs}
            onChange={(e) => setSilenceMs(Number(e.target.value))}
          />
        </label>
      </div>
      <div className="row">
        <button disabled={!sent || busy} onClick={() => void stream()}>
          Stream through noisy channel
        </button>
        <button className="secondary" disabled={!sent || busy} onClick={() => void decodeWhole()}>
          Decode whole clip
        </button>
        {!sent && <span className="muted">modulate a message first</span>}
      </div>
      {error && <p className="error">{error}</p>}
      {result && (
        <p className={result.crc_ok ? "ok" : "error"}>
          "{result.text}" · CRC {result.crc_ok ? "ok" : "FAILED"} · SNR {result.snr} dB
        </p>
      )}
      {noisy && <Waveform samples={noisy} color="#c2571a" />}
    </div>
  );
}
