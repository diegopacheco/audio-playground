import { useEffect, useState } from "react";
import { modulate } from "../api";
import { parsePcm16Wav } from "../audio/wav";
import type { ModemSettings, PcmAudio } from "../types";
import { SpectrogramView } from "./SpectrogramView";
import { Waveform } from "./Waveform";

interface Props {
  settings: ModemSettings;
  sent: PcmAudio | null;
  onSent: (audio: PcmAudio) => void;
}

export function SendPanel({ settings, sent, onSent }: Props) {
  const [text, setText] = useState("Hello from the speaker!");
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => () => {
    if (url) URL.revokeObjectURL(url);
  }, [url]);

  async function send(play: boolean) {
    setBusy(true);
    setError(null);
    try {
      const wav = await modulate(text, settings);
      onSent(parsePcm16Wav(wav));
      const next = URL.createObjectURL(new Blob([wav], { type: "audio/wav" }));
      setUrl(next);
      if (play) await new Audio(next).play();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="card">
      <h2>Send</h2>
      <textarea
        value={text}
        maxLength={512}
        rows={3}
        onChange={(e) => setText(e.target.value)}
        placeholder="Type a message to transmit"
      />
      <div className="row">
        <button disabled={busy || text.length === 0} onClick={() => void send(true)}>
          Modulate and play
        </button>
        <button className="secondary" disabled={busy || text.length === 0} onClick={() => void send(false)}>
          Modulate only
        </button>
        {url && (
          <a className="link" href={url} download="message.wav">
            Download WAV
          </a>
        )}
      </div>
      {error && <p className="error">{error}</p>}
      {sent && (
        <>
          <p className="muted">
            {(sent.samples.length / sent.sampleRate).toFixed(2)} s at {sent.sampleRate} Hz
          </p>
          <Waveform samples={sent.samples} color="#3b6fd8" />
          <SpectrogramView audio={sent} />
        </>
      )}
    </section>
  );
}
