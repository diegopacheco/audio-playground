import { useEffect, useState } from "react";
import { fetchTone, fetchWaveforms } from "./api";
import { headerHex, parseWavHeader } from "./audio/wavHeader";
import { pcmSamples } from "./audio/samples";
import { ToneForm } from "./components/ToneForm";
import { WaveCanvas } from "./components/WaveCanvas";
import { Player } from "./components/Player";
import { HeaderPanel } from "./components/HeaderPanel";
import type { Tone, ToneRequest, WaveformsInfo } from "./types";

const INITIAL: ToneRequest = {
  waveform: "sine",
  frequency: 440,
  duration_ms: 1000,
  amplitude: 0.5,
  sample_rate: 44100,
};

export function App() {
  const [options, setOptions] = useState<WaveformsInfo | null>(null);
  const [request, setRequest] = useState<ToneRequest>(INITIAL);
  const [tone, setTone] = useState<Tone | null>(null);
  const [hex, setHex] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchWaveforms()
      .then(setOptions)
      .catch((e: Error) => setError(e.message));
  }, []);

  useEffect(() => {
    return () => {
      if (tone) URL.revokeObjectURL(tone.url);
    };
  }, [tone]);

  async function generate() {
    setBusy(true);
    setError(null);
    try {
      const bytes = await fetchTone(request);
      const info = parseWavHeader(bytes);
      const url = URL.createObjectURL(new Blob([bytes], { type: "audio/wav" }));
      setHex(headerHex(bytes));
      setTone({ request, url, info, samples: pcmSamples(bytes, info.dataSize) });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main>
      <header>
        <h1>Tone Generator</h1>
        <p className="muted">Rust synthesizes the waveform and encodes a 16-bit PCM WAV by hand.</p>
      </header>
      {error && <div className="error">{error}</div>}
      {options && (
        <ToneForm
          value={request}
          waveforms={options.waveforms}
          sampleRates={options.sample_rates}
          busy={busy}
          onChange={setRequest}
          onSubmit={generate}
        />
      )}
      {tone && (
        <>
          <Player tone={tone} />
          <WaveCanvas samples={tone.samples} sampleRate={tone.info.sampleRate} frequency={tone.request.frequency} />
          <HeaderPanel info={tone.info} hex={hex} />
        </>
      )}
    </main>
  );
}
