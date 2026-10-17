import { useEffect, useMemo, useState } from "react";
import { fetchTestSignal, processWav } from "../api";
import type { RenderResult, VoiceSettings } from "../types";

interface Props {
  settings: VoiceSettings;
  onError: (message: string) => void;
}

const OFFLINE_RATE = 44100;
const OFFLINE_SECONDS = 4;

export function OfflinePanel({ settings, onError }: Props) {
  const [result, setResult] = useState<RenderResult | null>(null);
  const [busy, setBusy] = useState(false);

  const urls = useMemo(
    () => (result ? { original: URL.createObjectURL(result.original), processed: URL.createObjectURL(result.processed) } : null),
    [result],
  );

  useEffect(
    () => () => {
      if (urls) {
        URL.revokeObjectURL(urls.original);
        URL.revokeObjectURL(urls.processed);
      }
    },
    [urls],
  );

  async function render(original: Blob) {
    setBusy(true);
    try {
      setResult({ original, processed: await processWav(original, settings) });
    } catch (err) {
      onError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function renderTestVoice() {
    try {
      await render(await fetchTestSignal(OFFLINE_RATE, OFFLINE_SECONDS));
    } catch (err) {
      onError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <section className="card">
      <h2>Offline render</h2>
      <p className="hint">POST a WAV to /api/process with the current effect and get the processed WAV back.</p>
      <div className="buttons">
        <button className="primary" disabled={busy} onClick={renderTestVoice}>
          Render test voice
        </button>
        <label className={busy ? "file disabled" : "file"}>
          Upload WAV
          <input
            type="file"
            accept="audio/wav,.wav"
            disabled={busy}
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) {
                void render(file);
              }
              event.target.value = "";
            }}
          />
        </label>
      </div>
      {urls && (
        <div className="players">
          <div>
            <span>Original</span>
            <audio controls src={urls.original} />
          </div>
          <div>
            <span>Processed</span>
            <audio controls src={urls.processed} />
          </div>
          <a href={urls.processed} download={`voice-${settings.effect}.wav`}>
            Download processed WAV
          </a>
        </div>
      )}
    </section>
  );
}
